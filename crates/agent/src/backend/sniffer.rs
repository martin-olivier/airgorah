//! Native 802.11 capture engine — the in-house replacement for `airodump-ng`.
//!
//! A capture thread opens a raw `AF_PACKET` socket on the monitor interface,
//! hops (or parks on) the requested channels, and for every frame:
//!   * appends it to the live capture file (radiotap + 802.11, see [`super::pcap`]),
//!   * parses it and folds the result into the shared [`APS`] / [`UNLINKED_CLIENTS`]
//!     state that the GUI polls.
//!
//! Frame parsing is delegated to `radiotap` (to strip the radiotap header and read
//! signal/flags) and `libwifi` (to decode the 802.11 frame). The thread is stopped
//! by raising the `stop` flag and joining it (see [`super::scan`]).

use super::find_vendor;
use super::pcap::PcapWriter;
use super::scan::{get_aps, get_cap_ext, get_live_scan_path, get_unlinked_clients};

use airgorah_common::types::{AP, Client};

use libwifi::Addresses;
use libwifi::Frame;
use libwifi::frame::components::{DataHeader, ManagementHeader, RsnAkmSuite, StationInfo};

use std::collections::HashMap;
use std::ffi::CString;
use std::io;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

/// 2.4 GHz channels scanned when the band is enabled without a channel filter.
const CHANNELS_2_4: &[u32] = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13];

/// 5 GHz channels scanned when the band is enabled without a channel filter.
/// DFS channels are included; the card may refuse some depending on the
/// regulatory domain, in which case that hop is simply skipped.
const CHANNELS_5: &[u32] = &[
    36, 40, 44, 48, 52, 56, 60, 64, 100, 104, 108, 112, 116, 120, 124, 128, 132, 136, 140, 144,
    149, 153, 157, 161, 165,
];

/// How long to dwell on each channel while hopping.
const HOP_INTERVAL: Duration = Duration::from_millis(250);

/// Resolve the channels to visit from the enabled bands, or from an explicit
/// comma-separated channel filter when one is given.
pub fn build_channel_list(ghz_2_4: bool, ghz_5: bool, filter: Option<&str>) -> Vec<u32> {
    if let Some(filter) = filter {
        return filter
            .split_terminator(',')
            .filter_map(|c| c.parse::<u32>().ok())
            .collect();
    }

    let mut channels = Vec::new();
    if ghz_2_4 {
        channels.extend_from_slice(CHANNELS_2_4);
    }
    if ghz_5 {
        channels.extend_from_slice(CHANNELS_5);
    }
    channels
}

/// Capture-thread body. Runs until `stop` is raised.
pub fn run(iface: String, channels: Vec<u32>, stop: Arc<AtomicBool>) {
    let socket = match open_capture_socket(&iface) {
        Ok(socket) => socket,
        Err(e) => {
            log::error!("scan: could not open capture socket on {iface}: {e}");
            return;
        }
    };

    let mut pcap = match PcapWriter::create(get_live_scan_path() + get_cap_ext()) {
        Ok(pcap) => pcap,
        Err(e) => {
            log::error!("scan: could not create capture file: {e}");
            return;
        }
    };

    let hopping = channels.len() > 1;
    let mut chan_idx = 0;
    let mut current_channel = channels.first().copied().unwrap_or(0);
    if let Some(&channel) = channels.first() {
        set_channel(&iface, channel);
    }
    let mut last_hop = Instant::now();

    let mut buf = [0u8; 8192];

    while !stop.load(Ordering::Relaxed) {
        if hopping && last_hop.elapsed() >= HOP_INTERVAL {
            chan_idx = (chan_idx + 1) % channels.len();
            current_channel = channels[chan_idx];
            set_channel(&iface, current_channel);
            last_hop = Instant::now();
        }

        match recv_frame(&socket, &mut buf) {
            Ok(0) => {}
            Ok(n) => {
                let frame = &buf[..n];
                if let Err(e) = pcap.write_frame(frame) {
                    log::error!("scan: capture write failed: {e}");
                    break;
                }
                process_frame(frame, current_channel);
            }
            // A receive timeout (so the loop can hop / observe `stop`) surfaces as
            // EAGAIN/EWOULDBLOCK; EINTR is a benign interruption. Anything else is fatal.
            Err(e) => match e.raw_os_error() {
                Some(code)
                    if code == libc::EAGAIN || code == libc::EWOULDBLOCK || code == libc::EINTR => {
                }
                _ => {
                    log::error!("scan: capture recv failed: {e}");
                    break;
                }
            },
        }
    }

    log::info!("scan: capture thread on {iface} stopped");
}

// --- raw AF_PACKET socket ---------------------------------------------------

/// Open a raw `AF_PACKET` socket capturing every frame on `iface`, with a short
/// receive timeout so the capture loop can hop channels and observe the stop flag.
fn open_capture_socket(iface: &str) -> io::Result<OwnedFd> {
    let eth_p_all = (libc::ETH_P_ALL as u16).to_be();

    // SAFETY: standard socket(2) call; the returned fd is immediately wrapped in an
    // OwnedFd so it is closed on drop.
    let fd = unsafe { libc::socket(libc::AF_PACKET, libc::SOCK_RAW, eth_p_all as i32) };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }
    let socket = unsafe { OwnedFd::from_raw_fd(fd) };

    let ifindex = interface_index(iface)?;

    let mut addr: libc::sockaddr_ll = unsafe { std::mem::zeroed() };
    addr.sll_family = libc::AF_PACKET as u16;
    addr.sll_protocol = eth_p_all;
    addr.sll_ifindex = ifindex as i32;

    // SAFETY: bind(2) with a correctly sized sockaddr_ll.
    let ret = unsafe {
        libc::bind(
            socket.as_raw_fd(),
            &addr as *const libc::sockaddr_ll as *const libc::sockaddr,
            std::mem::size_of::<libc::sockaddr_ll>() as libc::socklen_t,
        )
    };
    if ret < 0 {
        return Err(io::Error::last_os_error());
    }

    let timeout = libc::timeval {
        tv_sec: 0,
        tv_usec: 100_000,
    };
    // SAFETY: setsockopt(2) with a correctly sized timeval.
    let ret = unsafe {
        libc::setsockopt(
            socket.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_RCVTIMEO,
            &timeout as *const libc::timeval as *const libc::c_void,
            std::mem::size_of::<libc::timeval>() as libc::socklen_t,
        )
    };
    if ret < 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(socket)
}

/// Resolve an interface name to its kernel index.
fn interface_index(iface: &str) -> io::Result<u32> {
    let name = CString::new(iface).map_err(|_| {
        io::Error::new(io::ErrorKind::InvalidInput, "interface name contains a nul")
    })?;
    // SAFETY: if_nametoindex(3) reads a valid C string; returns 0 on error.
    let index = unsafe { libc::if_nametoindex(name.as_ptr()) };
    if index == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(index)
}

/// Receive one frame, returning the number of bytes read (0 on an empty read).
fn recv_frame(socket: &OwnedFd, buf: &mut [u8]) -> io::Result<usize> {
    // SAFETY: recv(2) into a buffer of the given length.
    let n = unsafe {
        libc::recv(
            socket.as_raw_fd(),
            buf.as_mut_ptr() as *mut libc::c_void,
            buf.len(),
            0,
        )
    };
    if n < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(n as usize)
}

/// Tune the interface to a channel via `iw`. Failures (e.g. a regulatory-blocked
/// DFS channel) are non-fatal — the hop is just skipped.
fn set_channel(iface: &str, channel: u32) -> bool {
    std::process::Command::new("iw")
        .args(["dev", iface, "set", "channel", &channel.to_string()])
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

// --- frame processing -------------------------------------------------------

/// Strip radiotap, decode the 802.11 frame, and fold it into the shared state.
/// `current_channel` is the channel the card is tuned to, used as a fallback when
/// a beacon does not carry a DS-parameter channel (common on 5 GHz).
fn process_frame(raw: &[u8], current_channel: u32) {
    let (radiotap, body) = match radiotap::Radiotap::parse(raw) {
        Ok(parsed) => parsed,
        Err(_) => return,
    };

    if radiotap.flags.as_ref().is_some_and(|f| f.bad_fcs) {
        return;
    }
    let fcs_included = radiotap.flags.as_ref().is_some_and(|f| f.fcs);
    let signal = radiotap.antenna_signal.map(|s| s.value as i32);

    let frame = match libwifi::parse_frame(body, fcs_included) {
        Ok(frame) => frame,
        Err(_) => return,
    };

    match frame {
        Frame::Beacon(b) => update_ap(
            &b.header,
            b.capability_info,
            &b.station_info,
            signal,
            current_channel,
        ),
        Frame::ProbeResponse(b) => update_ap(
            &b.header,
            b.capability_info,
            &b.station_info,
            signal,
            current_channel,
        ),
        Frame::ProbeRequest(p) => record_probe(&p.header, &p.station_info, signal),
        Frame::Data(d) => record_data_client(&d.header, signal),
        Frame::QosData(d) => record_data_client(&d.header, signal),
        Frame::NullData(d) => record_data_client(&d.header, signal),
        Frame::QosNull(d) => record_data_client(&d.header, signal),
        _ => {}
    }
}

/// Fold a beacon / probe-response into the AP table.
fn update_ap(
    header: &ManagementHeader,
    capability_info: u16,
    info: &StationInfo,
    signal: Option<i32>,
    current_channel: u32,
) {
    let bssid = match header.bssid() {
        Some(mac) => mac.to_long_string(),
        None => return,
    };
    if !is_unicast(&bssid) {
        return;
    }

    let channel = info
        .channel()
        .map(i32::from)
        .unwrap_or(current_channel as i32);
    let band = band_of(channel);
    let privacy = classify_privacy(capability_info, info);
    let (essid, hidden) = essid_of(info);
    let power = signal.map(|s| s.to_string()).unwrap_or_default();
    let now = timestamp();

    let mut aps = get_aps();
    match aps.get_mut(&bssid) {
        Some(ap) => {
            // Keep a real ESSID we already learned rather than overwriting it with a
            // later hidden beacon.
            if !hidden {
                ap.essid = essid;
                ap.hidden = false;
            } else if ap.essid.is_empty() || ap.essid.starts_with("[Hidden] (length:") {
                ap.essid = essid;
                ap.hidden = true;
            }
            ap.channel = channel.to_string();
            ap.band = band;
            ap.privacy = privacy;
            if !power.is_empty() {
                ap.power = power;
            }
            ap.last_time_seen = now;
        }
        None => {
            aps.insert(
                bssid.clone(),
                AP {
                    essid,
                    bssid,
                    band,
                    channel: channel.to_string(),
                    power,
                    privacy,
                    hidden,
                    handshake: false,
                    saved_handshake: None,
                    first_time_seen: now.clone(),
                    last_time_seen: now,
                    clients: HashMap::new(),
                },
            );
        }
    }
}

/// Record a station seen in a probe request, along with the ESSID it probed for.
fn record_probe(header: &ManagementHeader, info: &StationInfo, signal: Option<i32>) {
    let station = header.address_2.to_long_string();
    if !is_unicast(&station) {
        return;
    }

    let probe = match info.essid() {
        Some(essid) if !essid.starts_with("<hidden") && !essid.is_empty() => Some(essid),
        _ => None,
    };

    record_client(&station, None, signal, probe);
}

/// Record a station seen exchanging data with an AP.
fn record_data_client(header: &DataHeader, signal: Option<i32>) {
    let fc = &header.frame_control;
    // ra() is address_1, ta() is address_2. In infrastructure BSS the AP is the
    // "distribution system" side, so the wireless station is the transmitter on
    // uplink and the receiver on downlink. Ad-hoc (neither bit) and WDS (both) are
    // ignored.
    let (station, bssid) = if fc.to_ds() && !fc.from_ds() {
        (header.ta(), header.ra())
    } else if fc.from_ds() && !fc.to_ds() {
        (header.ra(), header.ta())
    } else {
        return;
    };

    let station = station.to_long_string();
    let bssid = bssid.to_long_string();
    if !is_unicast(&station) {
        return;
    }

    record_client(&station, Some(&bssid), signal, None);
}

/// Insert or update a client, attaching it to a known AP when possible and
/// otherwise keeping it in the unlinked set (mirroring airodump's two lists).
fn record_client(station: &str, bssid: Option<&str>, signal: Option<i32>, probe: Option<String>) {
    let now = timestamp();
    let vendor = find_vendor(station);
    let power = signal.map(|s| s.to_string()).unwrap_or_default();

    let mut aps = get_aps();

    // Attach under the AP named by a data frame, or under the AP this station is
    // already listed with (so a later probe request updates the right record).
    let target = match bssid {
        Some(bssid) if aps.contains_key(bssid) => Some(bssid.to_string()),
        Some(_) | None => aps
            .iter()
            .find(|(_, ap)| ap.clients.contains_key(station))
            .map(|(bssid, _)| bssid.clone()),
    };

    if let Some(bssid) = target {
        let ap = aps.get_mut(&bssid).expect("target AP present");
        upsert_client(&mut ap.clients, station, &power, &vendor, &now, probe);
        drop(aps);
        get_unlinked_clients().remove(station);
        return;
    }

    drop(aps);
    upsert_client(
        &mut get_unlinked_clients(),
        station,
        &power,
        &vendor,
        &now,
        probe,
    );
}

/// Insert a new client record or refresh an existing one (bumping its packet
/// count, power, last-seen time, and merging in a newly probed ESSID).
fn upsert_client(
    clients: &mut HashMap<String, Client>,
    mac: &str,
    power: &str,
    vendor: &str,
    now: &str,
    probe: Option<String>,
) {
    match clients.get_mut(mac) {
        Some(client) => {
            client.packets = (client.packets.parse::<u64>().unwrap_or(0) + 1).to_string();
            if !power.is_empty() {
                client.power = power.to_string();
            }
            client.last_time_seen = now.to_string();
            if let Some(probe) = probe
                && !probe.is_empty()
                && !client.probes.split(", ").any(|p| p == probe)
            {
                if client.probes.is_empty() {
                    client.probes = probe;
                } else {
                    client.probes = format!("{}, {probe}", client.probes);
                }
            }
        }
        None => {
            clients.insert(
                mac.to_string(),
                Client {
                    mac: mac.to_string(),
                    packets: "1".to_string(),
                    power: power.to_string(),
                    first_time_seen: now.to_string(),
                    last_time_seen: now.to_string(),
                    vendor: vendor.to_string(),
                    probes: probe.unwrap_or_default(),
                },
            );
        }
    }
}

// --- field helpers ----------------------------------------------------------

/// Human-readable band label from a channel number (matches the previous CSV path).
fn band_of(channel: i32) -> String {
    if channel > 14 { "5 GHz" } else { "2.4 GHz" }.to_string()
}

/// Derive airgorah's single-token privacy label from the beacon's security IEs.
fn classify_privacy(capability_info: u16, info: &StationInfo) -> String {
    if let Some(rsn) = &info.rsn_information {
        if rsn.akm_suites.contains(&RsnAkmSuite::SAE) {
            return "WPA3".to_string();
        }
        return "WPA2".to_string();
    }
    if info.wpa_info.is_some() {
        return "WPA".to_string();
    }
    // Capability "Privacy" bit (bit 4) without RSN/WPA means legacy WEP.
    if capability_info & 0x0010 != 0 {
        return "WEP".to_string();
    }
    "OPN".to_string()
}

/// The ESSID and whether the network is hidden, in the format the GUI expects.
fn essid_of(info: &StationInfo) -> (String, bool) {
    match info.essid() {
        Some(essid) if !essid.starts_with("<hidden") && !essid.is_empty() => (essid, false),
        _ => {
            let length = info.ssid_length.unwrap_or(0);
            (format!("[Hidden] (length: {length})"), true)
        }
    }
}

/// A real, individually-addressed MAC (not broadcast, multicast, or all-zero).
fn is_unicast(mac: &str) -> bool {
    if mac == "00:00:00:00:00:00" {
        return false;
    }
    u8::from_str_radix(&mac[0..2], 16)
        .map(|first_octet| first_octet & 0x01 == 0)
        .unwrap_or(false)
}

/// Current local time in airodump's `YYYY-MM-DD HH:MM:SS` format.
fn timestamp() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

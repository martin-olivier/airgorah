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
use super::raw_socket;
use super::scan::{get_aps, get_cap_ext, get_live_scan_path, get_unlinked_clients};

use airgorah_common::channel::{CHANNELS_2_4, CHANNELS_5};
use airgorah_common::types::{AP, Client};

use libwifi::Addresses;
use libwifi::Frame;
use libwifi::frame::components::{DataHeader, ManagementHeader, RsnAkmSuite, StationInfo};

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::{Duration, Instant};

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
///
/// `channels` is shared and re-read every loop, so the GUI can change the bands or
/// channel filter (and the scanner retunes) without the thread being restarted.
///
/// `channel_out` is updated with the channel the card is tuned to on every hop, so
/// the GUI can display the channel the interface is currently listening on.
pub fn run(
    iface: String,
    channels: Arc<Mutex<Vec<u32>>>,
    channel_out: Arc<AtomicU32>,
    stop: Arc<AtomicBool>,
) {
    let socket = match raw_socket::open(&iface) {
        Ok(socket) => socket,
        Err(e) => {
            log::error!("scan: could not open capture socket on {iface}: {e}");
            return;
        }
    };
    // Short receive timeout so the loop can hop channels and observe `stop`.
    if let Err(e) = raw_socket::set_recv_timeout(&socket, 100) {
        log::error!("scan: could not set capture timeout on {iface}: {e}");
        return;
    }

    let mut pcap = match PcapWriter::create(get_live_scan_path() + get_cap_ext()) {
        Ok(pcap) => pcap,
        Err(e) => {
            log::error!("scan: could not create capture file: {e}");
            return;
        }
    };

    let mut chan_idx = 0;
    let mut current_channel = 0;
    let mut last_hop = Instant::now();

    // Tune to the first channel up front.
    if let Some(&channel) = channels.lock().unwrap().first() {
        current_channel = channel;
        channel_out.store(channel, Ordering::Relaxed);
        set_channel(&iface, channel);
    }

    let mut buf = [0u8; 8192];

    while !stop.load(Ordering::Relaxed) {
        // Re-read the (possibly just-updated) plan and retune if needed. The lock is
        // held only to decide; the slow `set_channel` runs without it.
        let hop_due = last_hop.elapsed() >= HOP_INTERVAL;
        let retune = {
            let channels = channels.lock().unwrap();
            plan_channel(&channels, &mut chan_idx, current_channel, hop_due)
        };
        if let Some(channel) = retune {
            current_channel = channel;
            channel_out.store(channel, Ordering::Relaxed);
            set_channel(&iface, channel);
            last_hop = Instant::now();
        }

        match raw_socket::recv(&socket, &mut buf) {
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

/// Tune the interface to a channel via `iw`. Failures (e.g. a regulatory-blocked
/// DFS channel) are non-fatal — the hop is just skipped.
fn set_channel(iface: &str, channel: u32) -> bool {
    std::process::Command::new("iw")
        .args(["dev", iface, "set", "channel", &channel.to_string()])
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

/// Decide the next channel to tune to from the live plan and where we sit in it.
///
/// Returns `Some(channel)` when the card should retune (advancing `chan_idx` for a
/// multi-channel plan), or `None` to stay put. Keeping the hop/park decision pure
/// lets it be unit-tested without a radio, and lets [`run`] re-read a plan the GUI
/// swapped under it — enabling a band or clearing the channel filter reshapes the
/// scan in place instead of restarting the thread:
///   * empty plan — nothing to tune to;
///   * single channel — park on it, retuning only if we are off it;
///   * many channels — advance to the next once the dwell time is up.
///
/// A stale `chan_idx` left over from a longer plan is folded back in bounds by the
/// modulo, so shrinking the plan never indexes out of range.
fn plan_channel(
    channels: &[u32],
    chan_idx: &mut usize,
    current: u32,
    hop_due: bool,
) -> Option<u32> {
    match channels.len() {
        0 => None,
        1 => (channels[0] != current).then_some(channels[0]),
        len => {
            if hop_due {
                *chan_idx = (*chan_idx + 1) % len;
                Some(channels[*chan_idx])
            } else {
                None
            }
        }
    }
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
                    pmkid: false,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_channel_list_from_bands() {
        assert_eq!(build_channel_list(true, false, None), CHANNELS_2_4.to_vec());
        assert_eq!(build_channel_list(false, true, None), CHANNELS_5.to_vec());
        assert!(build_channel_list(false, false, None).is_empty());

        let both = build_channel_list(true, true, None);
        assert_eq!(both.len(), CHANNELS_2_4.len() + CHANNELS_5.len());
        assert_eq!(both.first(), Some(&1));
    }

    #[test]
    fn build_channel_list_filter_overrides_bands() {
        // An explicit filter wins over the band toggles, and junk entries are dropped.
        assert_eq!(
            build_channel_list(true, true, Some("1,6,11")),
            vec![1, 6, 11]
        );
        assert_eq!(
            build_channel_list(true, false, Some("36,x,40")),
            vec![36, 40]
        );
        assert!(build_channel_list(true, false, Some("")).is_empty());
    }

    #[test]
    fn plan_channel_empty_stays_put() {
        let mut idx = 0;
        assert_eq!(plan_channel(&[], &mut idx, 0, true), None);
    }

    #[test]
    fn plan_channel_parked_holds_when_on_channel() {
        let mut idx = 0;
        assert_eq!(plan_channel(&[6], &mut idx, 6, true), None);
        assert_eq!(idx, 0);
    }

    #[test]
    fn plan_channel_parked_retunes_when_off_channel() {
        // Filter narrowed to a single channel while the card sits elsewhere.
        let mut idx = 0;
        assert_eq!(plan_channel(&[1], &mut idx, 6, false), Some(1));
    }

    #[test]
    fn plan_channel_hopping_waits_for_dwell() {
        let mut idx = 0;
        assert_eq!(plan_channel(&[1, 6, 11], &mut idx, 1, false), None);
        assert_eq!(idx, 0);
    }

    #[test]
    fn plan_channel_hopping_advances_when_due() {
        let mut idx = 0;
        assert_eq!(plan_channel(&[1, 6, 11], &mut idx, 1, true), Some(6));
        assert_eq!(idx, 1);
    }

    #[test]
    fn plan_channel_hopping_wraps_around() {
        let mut idx = 2;
        assert_eq!(plan_channel(&[1, 6, 11], &mut idx, 11, true), Some(1));
        assert_eq!(idx, 0);
    }

    #[test]
    fn plan_channel_stale_index_wraps_into_bounds() {
        // The plan shrank under a stale index; the modulo keeps the access valid.
        let mut idx = 5;
        assert_eq!(plan_channel(&[1, 6, 11], &mut idx, 6, true), Some(1));
        assert_eq!(idx, 0);
    }
}

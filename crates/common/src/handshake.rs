//! Native WPA handshake detection.
//!
//! Reads capture file(s) and reports which access points have a crackable WPA
//! 4-way handshake — the in-house replacement for shelling out to `aircrack-ng`.
//! Shared because both sides need it against different files: the agent scans the
//! root-owned live/old captures to flag APs while scanning, and the GUI inspects a
//! user-selected capture before offering it for decryption. Reading a capture and
//! parsing it needs no privilege, so the logic is identical on both sides.
//!
//! Only the classic libpcap format is read, with link type
//! `LINKTYPE_IEEE802_11_RADIOTAP` (127, what airgorah itself writes) or plain
//! `LINKTYPE_IEEE802_11` (105). Frames are decoded with `radiotap` + `libwifi`;
//! a data frame carrying an EAPOL key is classified into one of the four handshake
//! messages, and an AP is reported once a crackable combination has been seen.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use libwifi::Addresses;
use libwifi::Frame;
use libwifi::frame::EapolKey;
use libwifi::frame::components::{DataHeader, ManagementHeader, StationInfo};

const LINKTYPE_IEEE802_11: u32 = 105;
const LINKTYPE_IEEE802_11_RADIOTAP: u32 = 127;

/// Return `(bssid, essid)` for every AP that has a captured WPA 4-way handshake in
/// the given capture file(s). Unreadable or unparsable files are skipped.
pub fn get_handshakes<I, S>(paths: I) -> std::io::Result<Vec<(String, String)>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<Path>,
{
    let mut essids: HashMap<String, String> = HashMap::new();
    // Per (bssid, station): a bitmask of which handshake messages M1..M4 were seen.
    let mut messages: HashMap<(String, String), u8> = HashMap::new();

    for path in paths {
        if let Ok(data) = std::fs::read(path.as_ref()) {
            scan_capture(&data, &mut essids, &mut messages);
        }
    }

    // A crackable 4-way handshake needs M2 — the only carrier of the SNonce, plus a
    // MIC — together with an ANonce source (M1 or M3).
    let mut bssids: HashSet<String> = HashSet::new();
    for ((bssid, _station), seen) in &messages {
        let m1 = seen & 0b0001 != 0;
        let m2 = seen & 0b0010 != 0;
        let m3 = seen & 0b0100 != 0;
        if m2 && (m1 || m3) {
            bssids.insert(bssid.clone());
        }
    }

    Ok(bssids
        .into_iter()
        .map(|bssid| {
            let essid = essids
                .get(&bssid)
                .cloned()
                .unwrap_or_else(|| "hidden".to_string());
            (bssid, essid)
        })
        .collect())
}

/// Walk a single capture's frames, accumulating ESSIDs and handshake messages.
fn scan_capture(
    data: &[u8],
    essids: &mut HashMap<String, String>,
    messages: &mut HashMap<(String, String), u8>,
) {
    let Some(mut reader) = PcapReader::new(data) else {
        return;
    };
    let link_type = reader.link_type;
    if link_type != LINKTYPE_IEEE802_11 && link_type != LINKTYPE_IEEE802_11_RADIOTAP {
        return;
    }

    while let Some(record) = reader.next_frame() {
        let (body, fcs) = if link_type == LINKTYPE_IEEE802_11_RADIOTAP {
            let Ok((radiotap, rest)) = radiotap::Radiotap::parse(record) else {
                continue;
            };
            if radiotap.flags.as_ref().is_some_and(|f| f.bad_fcs) {
                continue;
            }
            (rest, radiotap.flags.as_ref().is_some_and(|f| f.fcs))
        } else {
            (record, false)
        };

        // The FCS flag is best-effort; fall back to parsing without it so a
        // mislabelled trailer never hides a handshake.
        let frame = match libwifi::parse_frame(body, fcs) {
            Ok(frame) => frame,
            Err(_) if fcs => match libwifi::parse_frame(body, false) {
                Ok(frame) => frame,
                Err(_) => continue,
            },
            Err(_) => continue,
        };

        match frame {
            Frame::Beacon(beacon) => record_essid(&beacon.header, &beacon.station_info, essids),
            Frame::ProbeResponse(resp) => record_essid(&resp.header, &resp.station_info, essids),
            Frame::Data(data) => record_eapol(&data.header, &data.eapol_key, messages),
            Frame::QosData(data) => record_eapol(&data.header, &data.eapol_key, messages),
            _ => {}
        }
    }
}

/// Remember an AP's ESSID from a beacon / probe response (ignoring hidden ones).
fn record_essid(
    header: &ManagementHeader,
    info: &StationInfo,
    essids: &mut HashMap<String, String>,
) {
    let Some(bssid) = header.bssid() else {
        return;
    };
    if let Some(essid) = info.essid()
        && !essid.is_empty()
        && !essid.starts_with("<hidden")
    {
        essids.entry(bssid.to_long_string()).or_insert(essid);
    }
}

/// Classify an EAPOL data frame and record which handshake message it is.
fn record_eapol(
    header: &DataHeader,
    eapol: &Option<EapolKey>,
    messages: &mut HashMap<(String, String), u8>,
) {
    let Some(key) = eapol else {
        return;
    };
    let Some(message) = classify_message(key.key_information) else {
        return;
    };

    // M1/M3 flow AP->station (from_ds); M2/M4 flow station->AP (to_ds).
    let fc = &header.frame_control;
    let (bssid, station) = if fc.to_ds() && !fc.from_ds() {
        (header.ra(), header.ta())
    } else if fc.from_ds() && !fc.to_ds() {
        (header.ta(), header.ra())
    } else {
        return;
    };

    let pair = (bssid.to_long_string(), station.to_long_string());
    *messages.entry(pair).or_insert(0) |= 1 << (message - 1);
}

/// Which 4-way handshake message (1..4) an EAPOL key frame is, from its Key
/// Information field, or `None` if it is not a pairwise handshake message.
fn classify_message(key_information: u16) -> Option<u8> {
    // Key Type bit (0x0008) distinguishes the pairwise 4-way from group rekeying.
    if key_information & 0x0008 == 0 {
        return None;
    }
    let ack = key_information & 0x0080 != 0;
    let mic = key_information & 0x0100 != 0;
    let secure = key_information & 0x0200 != 0;

    match (ack, mic, secure) {
        (true, false, _) => Some(1),     // ANonce, no MIC
        (false, true, false) => Some(2), // SNonce + MIC
        (true, true, _) => Some(3),      // ANonce + MIC, install/secure
        (false, true, true) => Some(4),  // MIC, secure
        _ => None,
    }
}

/// A minimal reader over the records of a classic libpcap file.
struct PcapReader<'a> {
    data: &'a [u8],
    offset: usize,
    big_endian: bool,
    link_type: u32,
}

impl<'a> PcapReader<'a> {
    /// Parse the 24-byte global header, detecting endianness from the magic.
    /// Returns `None` if the header is missing or not a libpcap magic.
    fn new(data: &'a [u8]) -> Option<Self> {
        if data.len() < 24 {
            return None;
        }
        let magic = u32::from_le_bytes(data[0..4].try_into().ok()?);
        let big_endian = match magic {
            // microsecond / nanosecond magics, same-endian as this reader
            0xa1b2_c3d4 | 0xa1b2_3c4d => false,
            // byte-swapped
            0xd4c3_b2a1 | 0x4d3c_b2a1 => true,
            _ => return None,
        };
        let link_type = read_u32(&data[20..24], big_endian);
        Some(Self {
            data,
            offset: 24,
            big_endian,
            link_type,
        })
    }

    /// Yield the next record's captured bytes, or `None` at the end (or on a
    /// truncated trailing record, which a live capture being read mid-write can
    /// present).
    fn next_frame(&mut self) -> Option<&'a [u8]> {
        if self.offset + 16 > self.data.len() {
            return None;
        }
        let incl_len = read_u32(
            &self.data[self.offset + 8..self.offset + 12],
            self.big_endian,
        ) as usize;
        let start = self.offset + 16;
        let end = start.checked_add(incl_len)?;
        if end > self.data.len() {
            return None;
        }
        self.offset = end;
        Some(&self.data[start..end])
    }
}

fn read_u32(bytes: &[u8], big_endian: bool) -> u32 {
    let arr = [bytes[0], bytes[1], bytes[2], bytes[3]];
    if big_endian {
        u32::from_be_bytes(arr)
    } else {
        u32::from_le_bytes(arr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_the_four_handshake_messages() {
        // Canonical Key Information values for a WPA2 4-way handshake.
        assert_eq!(classify_message(0x008a), Some(1)); // pairwise + ACK
        assert_eq!(classify_message(0x010a), Some(2)); // pairwise + MIC
        assert_eq!(classify_message(0x13ca), Some(3)); // pairwise + ACK + MIC + install/secure
        assert_eq!(classify_message(0x030a), Some(4)); // pairwise + MIC + secure
        // Group rekey (Key Type bit clear) and empty are not handshake messages.
        assert_eq!(classify_message(0x0382), None);
        assert_eq!(classify_message(0x0000), None);
    }

    /// The rule get_handshakes applies: M2 (SNonce+MIC) plus an ANonce source.
    fn is_complete(bits: u8) -> bool {
        let m1 = bits & 0b0001 != 0;
        let m2 = bits & 0b0010 != 0;
        let m3 = bits & 0b0100 != 0;
        m2 && (m1 || m3)
    }

    #[test]
    fn a_crackable_handshake_needs_m2_plus_anonce() {
        assert!(is_complete(0b0011)); // M1 + M2
        assert!(is_complete(0b0110)); // M2 + M3
        assert!(!is_complete(0b0010)); // M2 alone (no ANonce)
        assert!(!is_complete(0b0101)); // M1 + M3 (no SNonce)
        assert!(!is_complete(0b1000)); // M4 alone
    }

    fn push_record(data: &mut Vec<u8>, payload: &[u8]) {
        data.extend_from_slice(&0u32.to_le_bytes()); // ts_sec
        data.extend_from_slice(&0u32.to_le_bytes()); // ts_usec
        data.extend_from_slice(&(payload.len() as u32).to_le_bytes()); // incl_len
        data.extend_from_slice(&(payload.len() as u32).to_le_bytes()); // orig_len
        data.extend_from_slice(payload);
    }

    fn global_header(link_type: u32) -> Vec<u8> {
        let mut header = Vec::new();
        header.extend_from_slice(&0xa1b2_c3d4u32.to_le_bytes()); // magic (little-endian)
        header.extend_from_slice(&2u16.to_le_bytes()); // version major
        header.extend_from_slice(&4u16.to_le_bytes()); // version minor
        header.extend_from_slice(&0i32.to_le_bytes()); // thiszone
        header.extend_from_slice(&0u32.to_le_bytes()); // sigfigs
        header.extend_from_slice(&65535u32.to_le_bytes()); // snaplen
        header.extend_from_slice(&link_type.to_le_bytes()); // network
        header
    }

    #[test]
    fn pcap_reader_reads_records() {
        let mut data = global_header(LINKTYPE_IEEE802_11);
        push_record(&mut data, &[1, 2, 3]);
        push_record(&mut data, &[4, 5]);

        let mut reader = PcapReader::new(&data).expect("valid header");
        assert_eq!(reader.link_type, LINKTYPE_IEEE802_11);
        assert_eq!(reader.next_frame(), Some(&[1u8, 2, 3][..]));
        assert_eq!(reader.next_frame(), Some(&[4u8, 5][..]));
        assert_eq!(reader.next_frame(), None);
    }

    #[test]
    fn pcap_reader_drops_a_truncated_trailing_record() {
        // A live capture read mid-write can end with a partially-written record.
        let mut data = global_header(LINKTYPE_IEEE802_11_RADIOTAP);
        push_record(&mut data, &[1, 2, 3]);
        data.extend_from_slice(&0u32.to_le_bytes()); // ts_sec
        data.extend_from_slice(&0u32.to_le_bytes()); // ts_usec
        data.extend_from_slice(&10u32.to_le_bytes()); // incl_len claims 10 bytes
        data.extend_from_slice(&10u32.to_le_bytes());
        data.extend_from_slice(&[9, 9]); // ...but only 2 are present

        let mut reader = PcapReader::new(&data).expect("valid header");
        assert_eq!(reader.next_frame(), Some(&[1u8, 2, 3][..]));
        assert_eq!(reader.next_frame(), None); // truncated record ignored, no panic
    }

    #[test]
    fn rejects_non_pcap_data() {
        assert!(PcapReader::new(&[0u8; 4]).is_none()); // too short
        assert!(PcapReader::new(&[0u8; 24]).is_none()); // right size, bad magic
    }
}

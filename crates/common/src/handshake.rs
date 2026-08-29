//! Native WPA crackable-material detection (4-way handshake and PMKID).
//!
//! Reads capture file(s) and reports which access points have crackable WPA
//! material — a captured 4-way handshake and/or a PMKID. Shared because both sides
//! need it against different files: the agent scans the root-owned live/old captures
//! to flag APs while scanning, and the GUI inspects a user-selected capture before
//! offering it for decryption. Reading a capture and parsing it needs no
//! privilege, so the logic is identical on both sides.
//!
//! Only the classic libpcap format is read, with link type
//! `LINKTYPE_IEEE802_11_RADIOTAP` (127, what airgorah itself writes) or plain
//! `LINKTYPE_IEEE802_11` (105). Frames are decoded with `radiotap` + `libwifi`;
//! a data frame carrying an EAPOL key is classified into one of the four handshake
//! messages, and an AP is reported once a crackable combination has been seen. The
//! same message 1 also carries the AP's PMKID (in the RSN PMKID KDE of the key
//! data) when the AP volunteers it — a clientless capture that is crackable on its
//! own, so it is flagged independently of the 4-way handshake.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use libwifi::Addresses;
use libwifi::Frame;
use libwifi::frame::EapolKey;
use libwifi::frame::components::{DataHeader, ManagementHeader, StationInfo};

const LINKTYPE_IEEE802_11: u32 = 105;
const LINKTYPE_IEEE802_11_RADIOTAP: u32 = 127;

/// Crackable WPA material found for one access point in a capture.
///
/// `handshake` and `pmkid` are independent: either alone makes the AP crackable,
/// and both can be present in the same capture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Crackable {
    pub bssid: String,
    pub essid: String,
    pub handshake: bool,
    pub pmkid: bool,
}

/// Return a [`Crackable`] for every AP that yielded a WPA 4-way handshake and/or a
/// PMKID in the given capture file(s). Unreadable or unparsable files are skipped.
pub fn get_crackables<I, S>(paths: I) -> std::io::Result<Vec<Crackable>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<Path>,
{
    let mut essids: HashMap<String, String> = HashMap::new();
    // Per (bssid, station): a bitmask of which handshake messages M1..M4 were seen.
    let mut messages: HashMap<(String, String), u8> = HashMap::new();
    // BSSIDs whose message 1 carried a valid PMKID.
    let mut pmkids: HashSet<String> = HashSet::new();

    for path in paths {
        if let Ok(data) = std::fs::read(path.as_ref()) {
            scan_capture(&data, &mut essids, &mut messages, &mut pmkids);
        }
    }

    // A crackable 4-way handshake needs M2 — the only carrier of the SNonce, plus a
    // MIC — together with an ANonce source (M1 or M3).
    let mut handshakes: HashSet<String> = HashSet::new();
    for ((bssid, _station), seen) in &messages {
        let m1 = seen & 0b0001 != 0;
        let m2 = seen & 0b0010 != 0;
        let m3 = seen & 0b0100 != 0;
        if m2 && (m1 || m3) {
            handshakes.insert(bssid.clone());
        }
    }

    // Report every AP that produced either kind of material.
    let mut bssids: HashSet<String> = HashSet::new();
    bssids.extend(handshakes.iter().cloned());
    bssids.extend(pmkids.iter().cloned());

    Ok(bssids
        .into_iter()
        .map(|bssid| {
            let essid = essids
                .get(&bssid)
                .cloned()
                .unwrap_or_else(|| "hidden".to_string());
            let handshake = handshakes.contains(&bssid);
            let pmkid = pmkids.contains(&bssid);
            Crackable {
                bssid,
                essid,
                handshake,
                pmkid,
            }
        })
        .collect())
}

/// Return `(bssid, essid)` for every AP that has a captured WPA 4-way handshake in
/// the given capture file(s), ignoring PMKID-only APs. A thin view over
/// [`get_crackables`] for callers that only care about full handshakes.
pub fn get_handshakes<I, S>(paths: I) -> std::io::Result<Vec<(String, String)>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<Path>,
{
    Ok(get_crackables(paths)?
        .into_iter()
        .filter(|crackable| crackable.handshake)
        .map(|crackable| (crackable.bssid, crackable.essid))
        .collect())
}

/// Walk a single capture's frames, accumulating ESSIDs, handshake messages, and
/// the BSSIDs that leaked a PMKID.
fn scan_capture(
    data: &[u8],
    essids: &mut HashMap<String, String>,
    messages: &mut HashMap<(String, String), u8>,
    pmkids: &mut HashSet<String>,
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
            Frame::Data(data) => record_eapol(&data.header, &data.eapol_key, messages, pmkids),
            Frame::QosData(data) => record_eapol(&data.header, &data.eapol_key, messages, pmkids),
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

/// Classify an EAPOL data frame, record which handshake message it is, and note
/// the AP's PMKID when message 1 carries one.
fn record_eapol(
    header: &DataHeader,
    eapol: &Option<EapolKey>,
    messages: &mut HashMap<(String, String), u8>,
    pmkids: &mut HashSet<String>,
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

    let bssid = bssid.to_long_string();

    // Message 1 optionally advertises the AP's PMKID in the RSN PMKID KDE of its
    // key data. libwifi validates the KDE (OUI 00:0f:ac, type 4, non-zero PMKID)
    // and only returns it for message 1, so its presence alone flags the AP as
    // crackable — no client and no full handshake required.
    if key.pmkid().is_some() {
        pmkids.insert(bssid.clone());
    }

    let pair = (bssid, station.to_long_string());
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

    /// Build an 802.11 Data frame (from-DS) carrying an EAPOL-Key message 1, with
    /// an optional RSN PMKID KDE in its key data. `bssid`/`station` are the AP and
    /// client MACs; the AP is the transmitter (address 2) on this downlink frame.
    fn eapol_m1_frame(bssid: [u8; 6], station: [u8; 6], pmkid: Option<[u8; 16]>) -> Vec<u8> {
        let mut frame = Vec::new();

        // 802.11 MAC header (24 bytes): a from-DS data frame, so address 1 is the
        // destination station and address 2 is the transmitting AP (the BSSID).
        frame.extend_from_slice(&[0x08, 0x02]); // frame control: Data, subtype 0, from_ds
        frame.extend_from_slice(&[0x00, 0x00]); // duration
        frame.extend_from_slice(&station); // address 1 (RA): destination station
        frame.extend_from_slice(&bssid); // address 2 (TA): transmitting AP
        frame.extend_from_slice(&bssid); // address 3: BSSID
        frame.extend_from_slice(&[0x00, 0x00]); // sequence control

        // LLC/SNAP header announcing EAPOL (ethertype 0x888e).
        frame.extend_from_slice(&[0xaa, 0xaa, 0x03, 0x00, 0x00, 0x00, 0x88, 0x8e]);

        // Key data: an RSN PMKID KDE when a PMKID is provided, otherwise empty.
        let key_data = match pmkid {
            Some(pmkid) => {
                let mut kde = vec![0xdd, 0x14, 0x00, 0x0f, 0xac, 0x04]; // id, len, OUI, type 4
                kde.extend_from_slice(&pmkid);
                kde
            }
            None => Vec::new(),
        };

        // EAPOL-Key body (message 1: pairwise Key Type + Key ACK, no MIC).
        let mut eapol = Vec::new();
        eapol.push(0x02); // protocol version
        eapol.push(0x03); // packet type: EAPOL-Key
        eapol.extend_from_slice(&(95 + key_data.len() as u16).to_be_bytes()); // packet length
        eapol.push(0x02); // descriptor type: RSN
        eapol.extend_from_slice(&0x008au16.to_be_bytes()); // key information: message 1
        eapol.extend_from_slice(&0x0010u16.to_be_bytes()); // key length
        eapol.extend_from_slice(&1u64.to_be_bytes()); // replay counter
        eapol.extend_from_slice(&[0x11; 32]); // key nonce (ANonce)
        eapol.extend_from_slice(&[0x00; 16]); // key iv
        eapol.extend_from_slice(&0u64.to_be_bytes()); // key rsc
        eapol.extend_from_slice(&0u64.to_be_bytes()); // key id
        eapol.extend_from_slice(&[0x00; 16]); // key mic (absent in message 1)
        eapol.extend_from_slice(&(key_data.len() as u16).to_be_bytes()); // key data length
        eapol.extend_from_slice(&key_data);

        frame.extend_from_slice(&eapol);
        frame
    }

    #[test]
    fn detects_pmkid_in_message_one() {
        let bssid = [0x18, 0x86, 0x37, 0x1f, 0x30, 0x40];
        let station = [0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff];

        let mut essids = HashMap::new();
        let mut messages = HashMap::new();
        let mut pmkids = HashSet::new();

        let frame = eapol_m1_frame(bssid, station, Some([0x42; 16]));
        let mut data = global_header(LINKTYPE_IEEE802_11);
        push_record(&mut data, &frame);

        scan_capture(&data, &mut essids, &mut messages, &mut pmkids);

        // The AP's PMKID is recorded, and no full handshake is claimed from M1 alone.
        assert_eq!(pmkids.len(), 1);
        assert_eq!(messages.values().copied().collect::<Vec<_>>(), vec![0b0001]);
    }

    #[test]
    fn message_one_without_pmkid_is_not_flagged() {
        let bssid = [0x18, 0x86, 0x37, 0x1f, 0x30, 0x40];
        let station = [0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff];

        let mut essids = HashMap::new();
        let mut messages = HashMap::new();
        let mut pmkids = HashSet::new();

        // A message 1 whose key data has no PMKID KDE (empty key data).
        let frame = eapol_m1_frame(bssid, station, None);
        let mut data = global_header(LINKTYPE_IEEE802_11);
        push_record(&mut data, &frame);

        scan_capture(&data, &mut essids, &mut messages, &mut pmkids);

        assert!(pmkids.is_empty());
    }

    #[test]
    fn get_crackables_reports_a_pmkid_only_ap() {
        let bssid = [0x18, 0x86, 0x37, 0x1f, 0x30, 0x40];
        let station = [0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff];

        let mut data = global_header(LINKTYPE_IEEE802_11);
        push_record(&mut data, &eapol_m1_frame(bssid, station, Some([0x42; 16])));

        let mut path = std::env::temp_dir();
        path.push(format!("airgorah-pmkid-test-{}.cap", std::process::id()));
        std::fs::write(&path, &data).unwrap();

        let crackables = get_crackables([&path]).unwrap();
        std::fs::remove_file(&path).ok();

        // A lone PMKID makes the AP crackable even though there is no 4-way handshake.
        assert_eq!(crackables.len(), 1);
        assert!(crackables[0].pmkid);
        assert!(!crackables[0].handshake);
        // get_handshakes, which reports only 4-way handshakes, ignores it.
        let handshakes = get_handshakes([&path]).unwrap_or_default();
        assert!(handshakes.is_empty());
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

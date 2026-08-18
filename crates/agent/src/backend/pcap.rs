//! Minimal pcap writer for the native capture engine.
//!
//! The frames we pull off the monitor interface are radiotap-prefixed 802.11
//! frames, so the file is written with link-type
//! `LINKTYPE_IEEE802_11_RADIOTAP` (127) — the same format `airodump-ng`
//! produced. This keeps everything downstream (native handshake detection, the
//! capture accumulation, and the GUI capture export) working unchanged.
//!
//! Only the classic libpcap format is implemented (global header + per-record
//! header + raw bytes); that is all the readers need and it avoids pulling in a
//! libpcap dependency.

use std::fs::{File, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const LINKTYPE_IEEE802_11_RADIOTAP: u32 = 127;
const SNAPLEN: u32 = 65535;
const PCAP_MAGIC: u32 = 0xa1b2_c3d4;

/// Size of the libpcap global header (magic, versions, thiszone, sigfigs,
/// snaplen, network).
const GLOBAL_HEADER_LEN: usize = 24;

/// A writer appending captured frames to a libpcap file.
pub struct PcapWriter {
    file: BufWriter<File>,
}

impl PcapWriter {
    /// Create (truncating) a capture file at `path` and write the global header.
    pub fn create<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let mut file = BufWriter::new(File::create(path)?);

        file.write_all(&PCAP_MAGIC.to_ne_bytes())?;
        file.write_all(&2u16.to_ne_bytes())?; // version major
        file.write_all(&4u16.to_ne_bytes())?; // version minor
        file.write_all(&0i32.to_ne_bytes())?; // thiszone
        file.write_all(&0u32.to_ne_bytes())?; // sigfigs
        file.write_all(&SNAPLEN.to_ne_bytes())?; // snaplen
        file.write_all(&LINKTYPE_IEEE802_11_RADIOTAP.to_ne_bytes())?; // network
        file.flush()?;

        Ok(Self { file })
    }

    /// Append one captured frame (radiotap header included) as a pcap record.
    ///
    /// Flushes immediately: the live capture is read by handshake detection while
    /// the scan is still running, so records must hit the file as they arrive.
    pub fn write_frame(&mut self, frame: &[u8]) -> io::Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let len = frame.len() as u32;

        self.file.write_all(&(now.as_secs() as u32).to_ne_bytes())?;
        self.file.write_all(&now.subsec_micros().to_ne_bytes())?;
        self.file.write_all(&len.to_ne_bytes())?; // captured length
        self.file.write_all(&len.to_ne_bytes())?; // original length
        self.file.write_all(frame)?;
        self.file.flush()
    }
}

/// Append the packet records of `src` onto `dst`.
///
/// Both files are captures written by [`PcapWriter`], so they share an identical
/// global header, merging is therefore just a matter of appending records.
pub fn append_records(dst: &str, src: &str) -> io::Result<()> {
    let src_data = std::fs::read(src)?;
    if src_data.len() <= GLOBAL_HEADER_LEN {
        return Ok(());
    }

    let mut dst_file = OpenOptions::new().append(true).open(dst)?;
    dst_file.write_all(&src_data[GLOBAL_HEADER_LEN..])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> String {
        let mut path = std::env::temp_dir();
        path.push(format!("airgorah-pcap-test-{}-{name}", std::process::id()));
        path.to_str().unwrap().to_string()
    }

    #[test]
    fn append_records_concatenates_packets() {
        let dst = temp_path("dst.cap");
        let src = temp_path("src.cap");

        {
            let mut writer = PcapWriter::create(&dst).unwrap();
            writer.write_frame(&[1, 2, 3]).unwrap();
        }
        {
            let mut writer = PcapWriter::create(&src).unwrap();
            writer.write_frame(&[4, 5]).unwrap();
            writer.write_frame(&[6]).unwrap();
        }

        let dst_before = std::fs::read(&dst).unwrap();
        let src_bytes = std::fs::read(&src).unwrap();

        append_records(&dst, &src).unwrap();

        // dst keeps its own bytes and gains src's records (everything past the header).
        let dst_after = std::fs::read(&dst).unwrap();
        assert_eq!(
            dst_after.len(),
            dst_before.len() + (src_bytes.len() - GLOBAL_HEADER_LEN)
        );
        assert_eq!(&dst_after[..dst_before.len()], &dst_before[..]);
        assert_eq!(
            &dst_after[dst_before.len()..],
            &src_bytes[GLOBAL_HEADER_LEN..]
        );

        std::fs::remove_file(&dst).ok();
        std::fs::remove_file(&src).ok();
    }

    #[test]
    fn append_records_is_a_noop_for_a_header_only_source() {
        let dst = temp_path("dst-noop.cap");
        let src = temp_path("src-noop.cap");

        PcapWriter::create(&dst).unwrap().write_frame(&[9]).unwrap();
        PcapWriter::create(&src).unwrap(); // header only, no frames captured

        let before = std::fs::read(&dst).unwrap();
        append_records(&dst, &src).unwrap();
        let after = std::fs::read(&dst).unwrap();
        assert_eq!(before, after);

        std::fs::remove_file(&dst).ok();
        std::fs::remove_file(&src).ok();
    }
}

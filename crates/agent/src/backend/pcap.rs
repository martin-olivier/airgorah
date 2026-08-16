//! Minimal pcap writer for the native capture engine.
//!
//! The frames we pull off the monitor interface are radiotap-prefixed 802.11
//! frames, so the file is written with link-type
//! `LINKTYPE_IEEE802_11_RADIOTAP` (127) — the same format `airodump-ng`
//! produced. This keeps everything downstream (handshake detection via
//! `aircrack-ng`, `mergecap`, and the GUI capture export) working unchanged.
//!
//! Only the classic libpcap format is implemented (global header + per-record
//! header + raw bytes); that is all `aircrack-ng`/`mergecap` need and it avoids
//! pulling in a libpcap dependency.

use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const LINKTYPE_IEEE802_11_RADIOTAP: u32 = 127;
const SNAPLEN: u32 = 65535;
const PCAP_MAGIC: u32 = 0xa1b2_c3d4;

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

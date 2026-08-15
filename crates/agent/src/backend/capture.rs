use super::*;
use airgorah_common::handshake::get_handshakes;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// Update the handshake capture status of all APs by scanning the live/old
/// capture files. Runs agent-side because those files are root-owned.
pub fn update_handshakes() -> std::io::Result<()> {
    let paths: Vec<String> = [
        get_live_scan_path() + get_cap_ext(),
        get_old_scan_path() + get_cap_ext(),
    ]
    .into_iter()
    .filter(|path| Path::new(path).exists())
    .collect();

    if paths.is_empty() {
        return Ok(());
    }

    let handshakes = get_handshakes(&paths)?;

    let mut aps = get_aps();

    for (bssid, _) in handshakes {
        if let Some(ap) = aps.get_mut(&bssid) {
            ap.handshake = true;
        }
    }

    log::trace!("handshakes updated");

    Ok(())
}

/// Bytes per capture chunk. Kept well under the IPC frame cap: serde_json expands
/// a byte vector into a JSON array of integers (~3.6x).
const CHUNK_SIZE: u64 = 4 * 1024 * 1024;

/// Read one chunk of the accumulated capture at `offset`, returning the bytes and
/// whether it is the last chunk.
pub fn get_capture_chunk(offset: u64) -> std::io::Result<(Vec<u8>, bool)> {
    let mut file = std::fs::File::open(get_old_scan_path() + get_cap_ext())?;
    let len = file.metadata()?.len();

    let to_read = len.saturating_sub(offset).min(CHUNK_SIZE);
    let mut buf = vec![0u8; to_read as usize];

    file.seek(SeekFrom::Start(offset))?;
    file.read_exact(&mut buf)?;

    let last = offset + to_read >= len;

    Ok((buf, last))
}

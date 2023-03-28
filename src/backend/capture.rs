use super::*;
use crate::error::Error;
use crate::globals::*;

use std::process::Command;

/// Update the handshake capture status of all APs
pub fn update_handshakes() -> Result<(), Error> {
    let capture_output = Command::new("aircrack-ng")
        .args([
            &(LIVE_SCAN_PATH.to_string() + "-01.cap"),
            &(OLD_SCAN_PATH.to_string() + "-01.cap"),
        ])
        .output()?;

    let stdout = String::from_utf8_lossy(&capture_output.stdout).to_string();
    let lines = stdout.lines();
    let mut aps = get_aps();

    for data in lines {
        for (bssid, ap) in aps.iter_mut() {
            if data.contains(bssid) && data.contains("WPA (") && !data.contains("WPA (0 handshake)")
            {
                ap.handshake = true;
            }
        }
    }
    Ok(())
}

/// Save the current capture to a file
pub fn save_capture(path: &str) -> Result<(), Error> {
    std::fs::copy(OLD_SCAN_PATH.to_string() + "-01.cap", path)?;
    Ok(())
}

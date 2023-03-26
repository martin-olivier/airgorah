use crate::globals::*;
use super::*;
use std::process::Command;

/// Update the handshake capture status of all APs
pub fn update_handshakes() {
    let live_scan_output = Command::new("aircrack-ng")
        .args([&(LIVE_SCAN_PATH.to_string() + "-01.cap")])
        .output()
        .unwrap();

    let old_scan_output = Command::new("aircrack-ng")
        .args([&(OLD_SCAN_PATH.to_string() + "-01.cap")])
        .output()
        .unwrap();

    let mut stdout = String::from_utf8_lossy(&live_scan_output.stdout).to_string();
    stdout.push_str(&String::from_utf8_lossy(&old_scan_output.stdout));

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
}

/// Save the current capture to a file
pub fn save_capture(path: &str) {
    std::fs::copy(OLD_SCAN_PATH.to_string() + "-01.cap", path).ok();
}

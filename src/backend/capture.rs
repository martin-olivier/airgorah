use crate::globals::*;

pub fn update_handshakes() {
    let live_scan_output = std::process::Command::new("aircrack-ng")
        .args([&(LIVE_SCAN_PATH.to_string() + "-01.cap")])
        .output()
        .unwrap();

    let old_scan_output = std::process::Command::new("aircrack-ng")
        .args([&(OLD_SCAN_PATH.to_string() + "-01.cap")])
        .output()
        .unwrap();

    let mut stdout = String::from_utf8_lossy(&live_scan_output.stdout).to_string();
    stdout.push_str(&String::from_utf8_lossy(&old_scan_output.stdout).to_string());

    let mut lines = stdout.lines();
    let mut aps = super::get_aps();

    while let Some(data) = lines.next() {
        for (bssid, ap) in aps.iter_mut() {
            if data.contains(bssid) && data.contains("WPA (") && !data.contains("WPA (0 handshake)") {
                ap.handshake = true;
            }
        }
    }
}

pub fn save_capture(path: &str) {
    std::fs::copy(OLD_SCAN_PATH.to_string() + "-01.cap", path).ok();
}

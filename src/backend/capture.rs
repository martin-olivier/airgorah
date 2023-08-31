use super::*;
use crate::error::Error;
use crate::globals::*;

use regex::Regex;
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

pub fn get_handshakes(path: &str) -> Result<Vec<(String, String)>, Error> {
    let capture_output = Command::new("aircrack-ng").args([path]).output()?;

    let stdout = String::from_utf8_lossy(&capture_output.stdout).to_string();
    let re =
        Regex::new(r"(\d+)\s+([\w:]+)\s+([\w\s]*)\s+WPA \((\d+)\s+handshake(?:.*?)\)").unwrap();

    let mut handshakes = vec![];

    for caps in re.captures_iter(&stdout) {
        let bssid = caps[2].to_string();
        let essid = caps[3].trim().to_string();
        let handshake_count = caps[4].to_string();

        if handshake_count.parse::<i32>().unwrap_or(0) > 0 {
            handshakes.push((
                bssid,
                match essid.is_empty() {
                    true => "hidden".to_string(),
                    false => essid,
                },
            ));
        }
    }

    Ok(handshakes)
}

/// Save the current capture to a file
pub fn save_capture(path: &str) -> Result<(), Error> {
    std::fs::copy(OLD_SCAN_PATH.to_string() + "-01.cap", path)?;
    Ok(())
}

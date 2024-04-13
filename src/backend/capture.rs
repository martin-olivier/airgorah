use super::*;
use crate::globals::*;
use crate::types::*;

use regex::Regex;
use serde::Serialize;
use std::fs::File;
use std::io::Write;
use std::process::Command;

#[derive(Debug, Serialize)]
struct Report {
    pub access_points: Vec<AP>,
    pub unlinked_clients: Vec<Client>,
}

#[derive(thiserror::Error, Debug)]
pub enum CapError {
    #[error("Input/Output error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Regex error: {0}")]
    BadRegex(#[from] regex::Error),

    #[error("Json error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// Update the handshake capture status of all APs
pub fn update_handshakes() -> Result<(), CapError> {
    let handshakes = get_handshakes([
        &(LIVE_SCAN_PATH.to_string() + "-01.cap"),
        &(OLD_SCAN_PATH.to_string() + "-01.cap"),
    ])?;

    let mut aps = get_aps();

    for (bssid, _) in handshakes {
        if let Some(ap) = aps.get_mut(&bssid) {
            ap.handshake = true;
        }
    }

    log::trace!("handshakes updated");

    Ok(())
}

/// Get the access points infos of the handshakes contained in the capture file
pub fn get_handshakes<I, S>(args: I) -> Result<Vec<(String, String)>, CapError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let capture_output = Command::new("aircrack-ng").args(args).output()?;

    let output = String::from_utf8_lossy(&capture_output.stdout).to_string();

    let re = Regex::new(r"\s+(\d+)\s+([\w:]+)\s+(.*)\s+WPA \((\d+)\s+handshake.*\)")?;

    let mut handshakes = vec![];

    for line in output.lines() {
        let caps = match re.captures(line) {
            Some(caps) => caps,
            None => continue,
        };

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
pub fn save_capture(path: &str) -> Result<(), CapError> {
    std::fs::copy(OLD_SCAN_PATH.to_string() + "-01.cap", path)?;

    log::info!("capture saved to '{}'", path);

    Ok(())
}

/// Save the capture report to a file
pub fn save_report(path: &str) -> Result<(), CapError> {
    let access_points = get_aps().values().cloned().collect::<Vec<AP>>();
    let unlinked_clients = get_unlinked_clients()
        .values()
        .cloned()
        .collect::<Vec<Client>>();

    let report = Report {
        access_points,
        unlinked_clients,
    };

    let json_data = serde_json::to_string::<Report>(&report)?;

    let mut file = File::create(path)?;
    file.write_all(json_data.as_bytes())?;

    log::info!("report saved to '{}'", path);

    Ok(())
}

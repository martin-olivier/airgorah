//! Handshake detection via `aircrack-ng`.
//!
//! Shared because both sides need it against different files: the agent scans
//! the root-owned live/old captures to flag APs, while the GUI inspects
//! a user-selected capture before offering it for decryption. Reading a capture
//! and running `aircrack-ng` needs no privilege — only file access — so the logic
//! is identical on both sides.

use regex::Regex;
use std::process::Command;

/// Return `(bssid, essid)` for every AP that has at least one captured WPA
/// handshake in the given capture file(s).
pub fn get_handshakes<I, S>(args: I) -> std::io::Result<Vec<(String, String)>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let capture_output = Command::new("aircrack-ng").args(args).output()?;

    let output = String::from_utf8_lossy(&capture_output.stdout).to_string();

    // Pattern is a compile-time constant, so construction cannot fail.
    let re = Regex::new(r"\s+(\d+)\s+([\w:]+)\s+(.*)\s+WPA \((\d+)\s+handshake.*\)")
        .expect("valid handshake regex");

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

use crate::error::Error;

use std::process::{Command, Stdio};

const CRUNCH_LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
const CRUNCH_UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const CRUNCH_NUMBERS: &str = "0123456789";
const CRUNCH_SYMBOLS: &str = "!#$%/=?{}[]-*:;";

/// Launch a new terminal window to run aircrack-ng to decrypt a handshake with the specified wordlist
pub fn run_decrypt_wordlist_process(handshake: &str, bssid: &str, essid: &str, wordlist: &str) -> Result<(), Error> {
    let cmd = format!("aircrack-ng '{}' -b '{}' -w '{}' ; exec sh", handshake, bssid, wordlist);

    Command::new("gnome-terminal")
        .stdin(Stdio::piped())
        .args([
            "--hide-menubar",
            "--title",
            &format!("Handshake Decryption ({})", essid),
            "--",
            "sh",
            "-c",
            &cmd,
        ])
        .output()?;

    Ok(())
}

/// Launch a new terminal window to run aircrack-ng to decrypt a handshake using bruteforce
pub fn run_decrypt_bruteforce_process(handshake: &str, bssid: &str, essid: &str, low: bool, up: bool, num: bool, sym: bool) -> Result<(), Error> {
    let charset = format!("{}{}{}{}",
        match low {
            true => CRUNCH_LOWERCASE,
            false => "",
        },
        match up {
            true => CRUNCH_UPPERCASE,
            false => "",
        },
        match num {
            true => CRUNCH_NUMBERS,
            false => "",
        },
        match sym {
            true => CRUNCH_SYMBOLS,
            false => "",
        },
    );
    let cmd = format!("crunch 8 64 '{}' | aircrack-ng -w - -b '{}' '{}' ; exec sh", charset, bssid, handshake);

    Command::new("gnome-terminal")
        .stdin(Stdio::piped())
        .args([
            "--hide-menubar",
            "--title",
            &format!("Handshake Decryption ({})", essid),
            "--",
            "sh",
            "-c",
            &cmd,
        ])
        .output()?;

    Ok(())
}

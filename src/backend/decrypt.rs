use std::process::{Command, Stdio};

const CRUNCH_LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
const CRUNCH_UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const CRUNCH_NUMBERS: &str = "0123456789";
const CRUNCH_SYMBOLS: &str = " @!#$%^&*()-_+=~`[]{}|:;<>,.?/\\";

#[derive(thiserror::Error, Debug)]
pub enum DecryptError {
    #[error("Input/Output error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Launch a new terminal window to run aircrack-ng to decrypt a handshake with the specified wordlist
pub fn run_decrypt_wordlist_process(
    handshake: &str,
    bssid: &str,
    essid: &str,
    wordlist: &str,
) -> Result<(), DecryptError> {
    let title = format!("Handshake Decryption ({})", essid);

    Command::new("xterm")
        .stdin(Stdio::null())
        .args([
            "-hold",
            "-T",
            &title,
            "-e",
            "aircrack-ng",
            handshake,
            "-b",
            bssid,
            "-w",
            wordlist,
        ])
        .spawn()?;

    Ok(())
}

/// Launch a new terminal window to run aircrack-ng to decrypt a handshake using bruteforce
pub fn run_decrypt_bruteforce_process(
    handshake: &str,
    bssid: &str,
    essid: &str,
    low: bool,
    up: bool,
    num: bool,
    sym: bool,
) -> Result<(), DecryptError> {
    let charset = format!(
        "{}{}{}{}",
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
    let title = format!("Handshake Decryption ({})", essid);
    let cmd = format!(
        "crunch 8 64 '{}' | aircrack-ng -w - -b '{}' '{}'",
        charset, bssid, handshake
    );

    Command::new("xterm")
        .stdin(Stdio::null())
        .args(["-hold", "-T", &title, "-e", "sh", "-c", &cmd])
        .spawn()?;

    Ok(())
}

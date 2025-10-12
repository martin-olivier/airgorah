use crate::types::*;
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
    let title = format!("Handshake Decryption ({essid})");

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
    charset: &BruteforceCharset,
    min: u64,
    max: u64,
) -> Result<(), DecryptError> {
    let charset_str = match charset {
        BruteforceCharset::Params(settings) => {
            format!(
                "{}{}{}{}",
                match settings.lowercase {
                    true => CRUNCH_LOWERCASE,
                    false => "",
                },
                match settings.uppercase {
                    true => CRUNCH_UPPERCASE,
                    false => "",
                },
                match settings.numbers {
                    true => CRUNCH_NUMBERS,
                    false => "",
                },
                match settings.symbols {
                    true => CRUNCH_SYMBOLS,
                    false => "",
                },
            )
        }
        BruteforceCharset::Specific(custom) => custom.to_owned(),
    };
    let title = format!("Handshake Decryption ({essid})");
    let cmd =
        format!("crunch {min} {max} '{charset_str}' | aircrack-ng -w - -b '{bssid}' '{handshake}'");

    Command::new("xterm")
        .stdin(Stdio::null())
        .args(["-hold", "-T", &title, "-e", "sh", "-c", &cmd])
        .spawn()?;

    Ok(())
}

use std::process::{Command, Stdio};
use crate::error::Error;
use super::*;

const CRUNCH_LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
const CRUNCH_UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const CRUNCH_NUMBERS: &str = "0123456789";
const CRUNCH_SYMBOLS: &str = "!#$%/=?{}[]-*:;";

/// Get the terminal emulator
pub fn build_terminal(title: String, command: String) -> Result<Command, Error> {
    let err_msg = Error::new("No supported terminal found, please install one of the following: xfce4-terminal, gnome-terminal, konsole");

    if has_dependency("xfce4-terminal") {
        let mut process = Command::new("xfce4-terminal");
        process.stdin(Stdio::piped());
        process.args([
            "--hide-menubar",
            "--hide-toolbar",
            "--hide-scrollbar",
            "--hold",
            "-T",
            &title,
            "-e",
            &command,
        ]);
        return Ok(process);
    } else if has_dependency("gnome-terminal") {
        let mut process = Command::new("gnome-terminal");
        process.stdin(Stdio::piped());
        process.args([
            "--hide-menubar",
            "--title",
            &title,
            "--",
            "sh",
            "-c",
            &(command + " ; read"),
        ]);
        return Ok(process);
    } else if has_dependency("konsole") {
        let mut process = Command::new("konsole");
        process.stdin(Stdio::piped());
        process.args([
            "--hide-menubar",
            "--hide-tabbar",
            "--hold",
            "-p",
            &("title=".to_owned() + &title),
            "-e",
            "sh",
            "-c",
            &command,
        ]);
    }

    return Err(err_msg);
}

/// Launch a new terminal window to run aircrack-ng to decrypt a handshake with the specified wordlist
pub fn run_decrypt_wordlist_process(handshake: &str, bssid: &str, essid: &str, wordlist: &str) -> Result<(), Error> {
    let title = format!("Handshake Decryption ({})", essid);
    let cmd = format!(
        "aircrack-ng '{}' -b '{}' -w '{}'",
        handshake, bssid, wordlist
    );

    let mut process = build_terminal(title, cmd)?;

    std::thread::spawn(move || {
        process.output().unwrap();
    });

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
) -> Result<(), Error> {
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
        "sh -c \"crunch 8 64 '{}' | aircrack-ng -w - -b '{}' '{}'\"",
        charset, bssid, handshake
    );

    let mut process = build_terminal(title, cmd)?;

    std::thread::spawn(move || {
        process.output().unwrap();
    });

    Ok(())
}

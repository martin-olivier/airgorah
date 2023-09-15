use std::process::{Command, Stdio};

const CRUNCH_LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
const CRUNCH_UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const CRUNCH_NUMBERS: &str = "0123456789";
const CRUNCH_SYMBOLS: &str = "!#$%/=?{}[]-*:;";

/// Launch a new terminal window to run aircrack-ng to decrypt a handshake with the specified wordlist
pub fn run_decrypt_wordlist_process(
    handshake: &str,
    bssid: &str,
    essid: &str,
    wordlist: &str,
) {
    let cmd = format!(
        "aircrack-ng '{}' -b '{}' -w '{}'",
        handshake, bssid, wordlist
    );

    let mut process = Command::new("xfce4-terminal");
    process.stdin(Stdio::piped());
    process.args([
        "--hide-menubar",
        "--hide-toolbar",
        "--hide-scrollbar",
        "--hold",
        "-T",
        &format!("Handshake Decryption ({})", essid),
        "-e",
        &cmd,
    ]);

    std::thread::spawn(move || {
        process.output().unwrap();
    });
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
) {
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
    let cmd = format!(
        "sh -c \"crunch 8 64 '{}' | aircrack-ng -w - -b '{}' '{}'\"",
        charset, bssid, handshake
    );

    let mut process = Command::new("xfce4-terminal");
    process.stdin(Stdio::piped());
    process.args([
        "--hide-menubar",
        "--hide-toolbar",
        "--hide-scrollbar",
        "--hold",
        "-T",
        &format!("Handshake Decryption ({})", essid),
        "-e",
        &cmd,
    ]);

    std::thread::spawn(move || {
        process.output().unwrap();
    });
}

use crate::error::Error;

use std::process::{Command, Stdio};

/// Launch a new terminal window to run aircrack-ng to decrypt a handshake with the specified wordlist
pub fn run_decrypt_process(handshake: &str, wordlist: &str) -> Result<(), Error> {
    let cmd = format!("aircrack-ng '{}' -w '{}' ; exec sh", handshake, wordlist);

    Command::new("gnome-terminal")
        .stdin(Stdio::piped())
        .args([
            "--hide-menubar",
            "--title",
            "Handshake Decryption",
            "--",
            "sh",
            "-c",
            &cmd,
        ])
        .output()?;

    Ok(())
}

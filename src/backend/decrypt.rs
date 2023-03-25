use std::process::{Command, Stdio};

/// Launch a new terminal window to run aircrack-ng to decrypt a handshake with the specified wordlist
pub fn run_decrypt_process(handshake: &str, wordlist: &str) {
    let cmd = format!(
        "gnome-terminal --hide-menubar --title \"Handshake Decryption\" -- sh -c \"aircrack-ng '{}' -w '{}' ; exec sh\"",
        handshake,
        wordlist
    );

    Command::new("sh")
        .stdin(Stdio::piped())
        .args(["-c", &cmd])
        .output()
        .ok();
}

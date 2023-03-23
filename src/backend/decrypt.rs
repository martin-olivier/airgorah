use std::process::{Command, Stdio};

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

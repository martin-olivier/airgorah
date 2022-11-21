use crate::error::Error;
use crate::globals::*;
use crate::types::*;

pub fn is_capture_process() -> bool {
    CAPTURE_PROC.lock().unwrap().as_ref().is_some()
}

pub fn set_capture_process(ap: AP) -> Result<(), Error> {
    let iface = match super::get_iface().as_ref() {
        Some(res) => res.to_string(),
        None => return Err(Error::new("No interface set")),
    };

    if let Some(child) = CAPTURE_PROC.lock().unwrap().as_mut() {
        child.kill().unwrap();
        child.wait().unwrap();
    }

    std::fs::remove_file(CAPTURE_PATH.to_string() + "-01.cap").ok();

    let proc_args = vec![
        iface.as_str(),
        "-a",
        "--output-format",
        "cap",
        "-w",
        CAPTURE_PATH,
        "--write-interval",
        "1",
        "--channel",
        &ap.channel,
        "--bssid",
        &ap.bssid,
    ];

    let child = std::process::Command::new("airodump-ng")
        .args(proc_args)
        .stdout(std::process::Stdio::null())
        .spawn()?;

    CAPTURE_PROC.lock().unwrap().replace(child);

    Ok(())
}

pub fn stop_capture_process() {
    if let Some(child) = CAPTURE_PROC.lock().unwrap().as_mut() {
        child.kill().unwrap();
        child.wait().unwrap();
    }

    CAPTURE_PROC.lock().unwrap().take();

    std::fs::remove_file(CAPTURE_PATH.to_string() + "-01.cap").ok();
}

pub fn has_handshake() -> Result<bool, Error> {
    let output = std::process::Command::new("aircrack-ng")
        .args([&(CAPTURE_PATH.to_string() + "-01.cap")])
        .output()?;

    if !output.status.success() {
        return Ok(false);
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    
    Ok(stdout.contains("WPA (") && !stdout.contains("WPA (0 handshake)"))
}

pub fn save_capture(path: &str) {
    std::fs::copy(CAPTURE_PATH.to_string() + "-01.cap", path).ok();
}

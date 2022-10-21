use std::process::Command;
use crate::error::Error;
use crate::globals::*;

pub fn get_interfaces() -> Result<Vec<String>, Error> {
    let cmd = Command::new("sh")
        .args(["-c", "iw dev | awk \'$1==\"Interface\"{print $2}\'"])
        .output()?;

    if !cmd.status.success() {
        return Err(Error::new(
            "Failed to get interfaces",
        ))
    }

    let out = String::from_utf8(cmd.stdout).unwrap();

    Ok(out.split_terminator('\n').map(String::from).collect())
}

pub fn enable_monitor_mode(iface: &str) -> Result<String, Error> {
    let check_monitor_cmd = Command::new("iw")
        .args(["dev", iface, "info"])
        .output()?;

    if !check_monitor_cmd.status.success() {
        return Err(Error::new(
            "No such interface",
        ))
    }

    let check_monitor_output = String::from_utf8(check_monitor_cmd.stdout).unwrap();

    if check_monitor_output.contains("type monitor") {
        return Ok(iface.to_string());
    }

    let enable_monitor_cmd = Command::new("airmon-ng")
        .args(["start", iface])
        .output()?;

    if !enable_monitor_cmd.status.success() {
        return Err(Error::new(
            "Failed to enable monitor mode",
        ))
    }

    let check_monitor_cmd = Command::new("iw")
        .args(["dev", &(iface.to_string() + "mon"), "info"])
        .output()?;

    if !check_monitor_cmd.status.success() {
        return Err(Error::new(
            "Monitor mode has been enabled but the interface is not found",
        ))
    }

    let check_monitor_output = String::from_utf8(check_monitor_cmd.stdout)?;

    match check_monitor_output.contains("type monitor") {
        true => Ok(iface.to_string() + "mon"),
        false => Err(Error::new(
            "Failed to enable monitor mode",
        )),
    }
}

pub fn disable_monitor_mode(iface: &str) -> Result<(), Error> {
    let check_monitor_cmd = Command::new("iw")
        .args(["dev", iface, "info"])
        .output()?;

    if !check_monitor_cmd.status.success() {
        return Err(Error::new(
            "Failed to get current interface",
        ))
    }

    let check_monitor_output = String::from_utf8(check_monitor_cmd.stdout).unwrap();

    if !check_monitor_output.contains("type monitor") {
        return Ok(());
    }

    let disable_monitor_cmd = Command::new("airmon-ng")
        .args(["stop", iface])
        .output()?;

    match disable_monitor_cmd.status.success() {
        true => Ok(()),
        false => Err(Error::new(
            "Failed to disable monitor mode on current interface",
        )),
    }
}

pub fn get_iface() -> Option<String> {
    IFACE.lock().unwrap().clone()
}

pub fn set_iface(iface: String) {
    IFACE.lock().unwrap().replace(iface);
}
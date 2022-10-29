use crate::error::Error;
use crate::globals::*;
use std::process::Command;

pub fn get_interfaces() -> Result<Vec<String>, Error> {
    let cmd = Command::new("sh")
        .args(["-c", "iw dev | awk \'$1==\"Interface\"{print $2}\'"])
        .output()?;

    if !cmd.status.success() {
        return Err(Error::new("Failed to get interfaces"));
    }

    let out = String::from_utf8(cmd.stdout)?;

    Ok(out.split_terminator('\n').map(String::from).collect())
}

pub fn is_5ghz_supported(iface: &str) -> Result<bool, Error> {
    let check_band_cmd = Command::new("iwlist").args([iface, "freq"]).output()?;

    if !check_band_cmd.status.success() {
        return Err(Error::new("No such interface"));
    }

    let check_band_output = String::from_utf8(check_band_cmd.stdout)?;

    if check_band_output.contains("Channel 36 : 5.18 GHz") {
        return Ok(true);
    }

    Ok(false)
}

pub fn is_monitor_mode(iface: &str) -> Result<bool, Error> {
    let check_monitor_cmd = Command::new("iw").args(["dev", iface, "info"]).output()?;

    if !check_monitor_cmd.status.success() {
        return Err(Error::new("No such interface"));
    }

    let check_monitor_output = String::from_utf8(check_monitor_cmd.stdout)?;

    if check_monitor_output.contains("type monitor") {
        return Ok(true);
    }

    Ok(false)
}

pub fn enable_monitor_mode(iface: &str) -> Result<String, Error> {
    kill_network_manager().ok();

    if is_monitor_mode(iface)? {
        return Ok(iface.to_string());
    }

    let old_interface_list = get_interfaces()?;
    let enable_monitor_cmd = Command::new("airmon-ng").args(["start", iface]).output()?;

    if !enable_monitor_cmd.status.success() {
        return Err(Error::new("Failed to enable monitor mode"));
    }

    match is_monitor_mode(&(iface.to_string() + "mon")) {
        Ok(res) => {
            match res {
                true => Ok(iface.to_string() + "mon"),
                false => Err(Error::new("Failed to enable monitor mode")),
            }
        },
        Err(_) => {
            let new_interface_list = get_interfaces()?;

            for iface_it in new_interface_list {
                if !old_interface_list.contains(&iface_it) && is_monitor_mode(&iface_it)? {
                    return Ok(iface_it);
                }
            }

            Err(Error::new(
                "Monitor mode has been enabled but the new interface has not been found",
            ))
        }
    }
}

pub fn disable_monitor_mode(iface: &str) -> Result<(), Error> {
    let check_monitor_cmd = Command::new("iw").args(["dev", iface, "info"]).output()?;

    if !check_monitor_cmd.status.success() {
        return Err(Error::new("Failed to get current interface"));
    }

    let check_monitor_output = String::from_utf8(check_monitor_cmd.stdout).unwrap();

    if !check_monitor_output.contains("type monitor") {
        return Ok(());
    }

    let disable_monitor_cmd = Command::new("airmon-ng").args(["stop", iface]).output()?;

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

pub fn kill_network_manager() -> Result<(), Error> {
    Command::new("airmon-ng").args(["check", "kill"]).output()?;

    Ok(())
}

pub fn restore_network_manager() -> Result<(), Error> {
    Command::new("service").args(["NetworkManager", "restart"]).output()?;
    Command::new("service").args(["network-manager", "restart"]).output()?;
    Command::new("service").args(["wpa-supplicant", "restart"]).output()?;

    Ok(())
}
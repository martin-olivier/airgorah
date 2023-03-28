use super::*;
use crate::error::Error;
use crate::globals::*;
use std::process::Command;

/// Get the available interfaces
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

/// Check if an interface supports 5GHz
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

/// Check if an interface is in monitor mode
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

// Set the MAC address of an interface
pub fn set_mac_address(iface: &str) -> Result<(), Error> {
    if !is_monitor_mode(iface)? {
        return Err(Error::new(
            "Can't change MAC address, interface is not in monitor mode",
        ));
    }

    Command::new("ifconfig").args([iface, "down"]).output()?;

    let success = match get_settings().mac_address.as_str() {
        "random" => {
            Command::new("macchanger").args(["-A", iface]).output()?;
            true
        }
        "default" => {
            Command::new("macchanger").args(["-p", iface]).output()?;
            true
        }
        mac => Command::new("macchanger")
            .args(["-m", mac, iface])
            .output()?
            .status
            .success(),
    };

    Command::new("ifconfig").args([iface, "up"]).output()?;

    if !success {
        return Err(Error::new(
            "The MAC address is invalid. Change the value in the settings page.",
        ));
    }

    Ok(())
}

/// enable monitor mode on an interface
pub fn enable_monitor_mode(iface: &str) -> Result<String, Error> {
    kill_network_manager()?;

    if is_monitor_mode(iface)? {
        return Ok(iface.to_string());
    }

    let old_interface_list = get_interfaces()?;
    let enable_monitor_cmd = Command::new("airmon-ng").args(["start", iface]).output()?;

    if !enable_monitor_cmd.status.success() {
        return Err(Error::new("Failed to enable monitor mode"));
    }

    match is_monitor_mode(&(iface.to_string() + "mon")) {
        Ok(res) => match res {
            true => Ok(iface.to_string() + "mon"),
            false => Err(Error::new("Failed to enable monitor mode")),
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

/// disable monitor mode on an interface
pub fn disable_monitor_mode(iface: &str) -> Result<(), Error> {
    let check_monitor_cmd = Command::new("iw").args(["dev", iface, "info"]).output()?;

    if !check_monitor_cmd.status.success() {
        return Err(Error::new("Failed to get current interface"));
    }

    let check_monitor_output = String::from_utf8(check_monitor_cmd.stdout)?;

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

/// Get the current interface
pub fn get_iface() -> Option<String> {
    IFACE.lock().unwrap().clone()
}

/// Set the current interface
pub fn set_iface(iface: String) {
    IFACE.lock().unwrap().replace(iface);
}

/// Kill the network manager to avoid channel hopping conflicts
pub fn kill_network_manager() -> Result<(), Error> {
    if get_settings().kill_network_manager {
        Command::new("airmon-ng").args(["check", "kill"]).output()?;
    }

    Ok(())
}

/// Restore the network manager
pub fn restore_network_manager() -> Result<(), Error> {
    if !get_settings().kill_network_manager {
        return Ok(());
    }

    Command::new("service")
        .args(["NetworkManager", "restart"])
        .output()?;
    Command::new("service")
        .args(["network-manager", "restart"])
        .output()?;
    Command::new("service")
        .args(["wpa-supplicant", "restart"])
        .output()?;

    Ok(())
}

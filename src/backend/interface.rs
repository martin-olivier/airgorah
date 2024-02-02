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
    let phy_path = format!("/sys/class/net/{}/phy80211", iface);

    let phy_link = std::fs::read_link(phy_path)?;

    let phy_name = match phy_link.file_name() {
        Some(name) => name,
        None => return Err(Error::new("phy parsing error")),
    };

    let phy_name_str = match phy_name.to_str() {
        Some(name) => name,
        None => return Err(Error::new("phy parsing error")),
    };

    let check_band_cmd = Command::new("iw")
        .args(["phy", phy_name_str, "info"])
        .output()?;

    if !check_band_cmd.status.success() {
        return Err(Error::new(&format!("{}: No such phy", phy_name_str)));
    }

    let check_band_output = String::from_utf8(check_band_cmd.stdout)?;

    if check_band_output.contains("5200 MHz") {
        return Ok(true);
    }

    Ok(false)
}

/// Check if an interface is in monitor mode
pub fn is_monitor_mode(iface: &str) -> Result<bool, Error> {
    let check_monitor_cmd = Command::new("iw").args(["dev", iface, "info"]).output()?;

    if !check_monitor_cmd.status.success() {
        return Err(Error::new(&format!("{}: No such interface", iface)));
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

    Command::new("ip")
        .args(["link", "set", "dev", iface, "down"])
        .output()?;

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

    Command::new("ip")
        .args(["link", "set", "dev", iface, "up"])
        .output()?;

    if !success {
        return Err(Error::new(
            "The MAC address is invalid. Change the value in the settings page.",
        ));
    }

    log::info!(
        "{}: MAC address changed to {}",
        iface,
        get_settings().mac_address
    );

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
        return Err(Error::new(&format!(
            "Could not enable monitor mode on \"{}\"",
            iface
        )));
    }

    log::info!("{}: monitor mode enabled", iface);

    if let Ok(true) = is_monitor_mode(iface) {
        return Ok(iface.to_string());
    }

    match is_monitor_mode(&(iface.to_string() + "mon")) {
        Ok(res) => match res {
            true => Ok(iface.to_string() + "mon"),
            false => Err(Error::new("Interface is still in managed mode")),
        },
        Err(_) => {
            let new_interface_list = get_interfaces()?;

            for iface_it in new_interface_list {
                if !old_interface_list.contains(&iface_it) && is_monitor_mode(&iface_it)? {
                    return Ok(iface_it);
                }
            }

            Err(Error::new(
                &format!("Monitor mode has been enabled on \"{}\", but the new interface name could not been found", iface)
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

    log::info!("{}: monitor mode disabled", iface);

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

/// List of services that can interfere with the app on the management of wireless cards
const INTERFERENCE_SERVICES: [&str; 19] = [
    "wpa_action",
    "wpa_supplicant",
    "wpa_cli",
    "dhclient",
    "ifplugd",
    "dhcdbd",
    "dhcpcd",
    "udhcpc",
    "NetworkManager",
    "knetworkmanager",
    "avahi-autoipd",
    "avahi-daemon",
    "wlassistant",
    "wifibox",
    "net_applet",
    "wicd-daemon",
    "wicd-client",
    "iwd",
    "hostapd",
];

/// Service manager related information
struct ServiceManager {
    cmd: String,
    status: String,
    start: String,
    stop: String,
}

impl ServiceManager {
    fn systemd() -> ServiceManager {
        ServiceManager {
            cmd: "systemctl".to_string(),
            status: "is-active".to_string(),
            start: "start".to_string(),
            stop: "stop".to_string(),
        }
    }

    fn runit_sv() -> ServiceManager {
        ServiceManager {
            cmd: "sv".to_string(),
            status: "status".to_string(),
            start: "up".to_string(),
            stop: "down".to_string(),
        }
    }
}

/// Kill the network manager to avoid channel hopping conflicts
pub fn kill_network_manager() -> Result<(), Error> {
    if get_settings().kill_network_manager {
        let service_manager = if has_dependency("systemctl") {
            ServiceManager::systemd()
        } else if has_dependency("sv") {
            ServiceManager::runit_sv()
        } else {
            return Err(Error::new("No service manager found, 'systemd' or 'runit' is required"));
        };

        for service in INTERFERENCE_SERVICES {
            let is_service_running = Command::new(&service_manager.cmd)
                .args([&service_manager.status, service])
                .output()?;

            if is_service_running.status.success() {
                Command::new(&service_manager.cmd)
                    .args([&service_manager.stop, service])
                    .output()?;

                SERVICES_TO_RESTORE.lock().unwrap().push(service.to_string());

                log::warn!("killed '{}'", service);
            }
        }
    }

    Ok(())
}

/// Restore the network manager
pub fn restore_network_manager() -> Result<(), Error> {
    if !get_settings().kill_network_manager {
        return Ok(());
    }

    let service_manager = if has_dependency("systemctl") {
        ServiceManager::systemd()
    } else if has_dependency("sv") {
        ServiceManager::runit_sv()
    } else {
        return Err(Error::new("No service manager found, 'systemd' or 'runit' is required"));
    };

    let services_to_restore: Vec<_> = SERVICES_TO_RESTORE.lock().unwrap().drain(..).collect();

    for service in services_to_restore {
        Command::new(&service_manager.cmd)
            .args([&service_manager.start, &service])
            .output()?;

        log::warn!("restored '{}'", service);
    }

    Ok(())
}

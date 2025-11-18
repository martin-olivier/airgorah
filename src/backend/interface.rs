use super::*;
use crate::globals::*;
use std::process::{Command, Stdio};

#[derive(thiserror::Error, Debug)]
pub enum IfaceError {
    #[error("Input/Output error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to retreive interfaces list")]
    IfaceList,

    #[error("Utf8 conversion error")]
    Utf8Error(#[from] std::string::FromUtf8Error),

    #[error("PHY parsing error")]
    PhyParsing,

    #[error("PHY '{0}' could not be found")]
    PhyNotFound(String),

    #[error("Interface '{0}' could not be found")]
    IfaceNotFound(String),

    #[error("Could not change MAC address: interface not in monitor mode")]
    IfaceNotMonitor,

    #[error("MAC address is invalid: change its value in the settings page.")]
    InvalidMac,

    #[error("Could not enable monitor mode on '{0}':\n{1}")]
    MonitorFailed(String, String),

    #[error("Could not disable monitor mode on '{0}':\n{1}")]
    ManagedFailed(String, String),

    #[error("systemctl is required to kill/restore network managers")]
    MissingSystemd,
}

/// Get the available interfaces
pub fn get_interfaces() -> Result<Vec<String>, IfaceError> {
    let cmd = Command::new("sh")
        .args(["-c", "iw dev | awk \'$1==\"Interface\"{print $2}\'"])
        .output()?;

    if !cmd.status.success() {
        return Err(IfaceError::IfaceList);
    }

    let out = String::from_utf8(cmd.stdout)?;

    Ok(out.split_terminator('\n').map(String::from).collect())
}

/// Check if an interface supports 5GHz
pub fn is_5ghz_supported(iface: &str) -> Result<bool, IfaceError> {
    let phy_path = format!("/sys/class/net/{iface}/phy80211");

    let phy_link = std::fs::read_link(phy_path)?;

    let phy_name = match phy_link.file_name() {
        Some(name) => name,
        None => return Err(IfaceError::PhyParsing),
    };

    let phy_name_str = match phy_name.to_str() {
        Some(name) => name,
        None => return Err(IfaceError::PhyParsing),
    };

    let check_band_cmd = Command::new("iw")
        .args(["phy", phy_name_str, "info"])
        .output()?;

    if !check_band_cmd.status.success() {
        return Err(IfaceError::PhyNotFound(phy_name_str.to_string()));
    }

    let check_band_output = String::from_utf8(check_band_cmd.stdout)?;

    if check_band_output.contains("5200 MHz") || check_band_output.contains("5200.0 MHz") {
        return Ok(true);
    }

    Ok(false)
}

/// Check if an interface is in monitor mode
pub fn is_monitor_mode(iface: &str) -> Result<bool, IfaceError> {
    let check_monitor_cmd = Command::new("iw").args(["dev", iface, "info"]).output()?;

    if !check_monitor_cmd.status.success() {
        return Err(IfaceError::IfaceNotFound(iface.to_string()));
    }

    let check_monitor_output = String::from_utf8(check_monitor_cmd.stdout)?;

    if check_monitor_output.contains("type monitor") {
        return Ok(true);
    }

    Ok(false)
}

// Set the MAC address of an interface
pub fn set_mac_address(iface: &str) -> Result<(), IfaceError> {
    if !is_monitor_mode(iface)? {
        return Err(IfaceError::IfaceNotMonitor);
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
        return Err(IfaceError::InvalidMac);
    }

    log::info!(
        "{}: MAC address changed to {}",
        iface,
        get_settings().mac_address
    );

    Ok(())
}

/// enable monitor mode on an interface
pub fn enable_monitor_mode(iface: &str) -> Result<String, IfaceError> {
    if is_monitor_mode(iface)? {
        *IFACE_WAS_MONITOR.lock().unwrap() = true;
    }

    kill_network_manager()?;

    if is_monitor_mode(iface)? {
        return Ok(iface.to_string());
    }

    let old_interface_list = get_interfaces()?;

    let yes_pipe = Command::new("yes").stdout(Stdio::piped()).spawn()?;
    let enable_monitor_cmd = Command::new("airmon-ng")
        .args(["start", iface])
        .stdin(yes_pipe.stdout.unwrap())
        .output()?;

    if !enable_monitor_cmd.status.success() {
        return Err(IfaceError::MonitorFailed(
            iface.to_string(),
            String::from_utf8(enable_monitor_cmd.stdout).unwrap_or_default(),
        ));
    }

    log::info!("{iface}: monitor mode enabled");

    if let Ok(true) = is_monitor_mode(iface) {
        return Ok(iface.to_string());
    }

    match is_monitor_mode(&(iface.to_string() + "mon")) {
        Ok(true) => Ok(iface.to_string() + "mon"),
        Ok(false) => Err(IfaceError::MonitorFailed(
            iface.to_string(),
            String::from_utf8(enable_monitor_cmd.stdout).unwrap_or_default(),
        )),
        Err(_) => {
            for iface in get_interfaces()? {
                if !old_interface_list.contains(&iface) && is_monitor_mode(&iface)? {
                    return Ok(iface);
                }
            }

            Err(IfaceError::MonitorFailed(
                iface.to_string(),
                String::from_utf8(enable_monitor_cmd.stdout).unwrap_or_default(),
            ))
        }
    }
}

/// disable monitor mode on an interface
pub fn disable_monitor_mode(iface: &str) -> Result<(), IfaceError> {
    let check_monitor_cmd = Command::new("iw").args(["dev", iface, "info"]).output()?;

    if !check_monitor_cmd.status.success() {
        return Err(IfaceError::IfaceNotFound(iface.to_string()));
    }

    let check_monitor_output = String::from_utf8(check_monitor_cmd.stdout)?;

    if !check_monitor_output.contains("type monitor") {
        return Ok(());
    }

    let mut iface_was_monitor = IFACE_WAS_MONITOR.lock().unwrap();
    if *iface_was_monitor {
        *iface_was_monitor = false;
        return Ok(());
    }

    let disable_monitor_cmd = Command::new("airmon-ng").args(["stop", iface]).output()?;

    log::info!("{iface}: monitor mode disabled");

    match disable_monitor_cmd.status.success() {
        true => Ok(()),
        false => Err(IfaceError::ManagedFailed(
            iface.to_string(),
            String::from_utf8(disable_monitor_cmd.stdout).unwrap_or_default(),
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

/// Kill the network manager to avoid channel hopping conflicts
pub fn kill_network_manager() -> Result<(), IfaceError> {
    if !get_settings().kill_network_manager {
        return Ok(());
    }

    if !has_dependency("systemctl") {
        return Err(IfaceError::MissingSystemd);
    }

    for service in INTERFERENCE_SERVICES {
        let is_service_running = Command::new("systemctl")
            .args(["is-active", service])
            .output()?;

        if is_service_running.status.success() {
            Command::new("systemctl").args(["stop", service]).output()?;

            SERVICES_TO_RESTORE
                .lock()
                .unwrap()
                .push(service.to_string());

            log::warn!("killed '{service}'");
        }
    }

    Ok(())
}

/// Restore the network manager
pub fn restore_network_manager() -> Result<(), IfaceError> {
    if !get_settings().kill_network_manager {
        return Ok(());
    }

    if !has_dependency("systemctl") {
        return Err(IfaceError::MissingSystemd);
    };

    let services_to_restore: Vec<_> = SERVICES_TO_RESTORE.lock().unwrap().drain(..).collect();

    for service in services_to_restore {
        Command::new("systemctl")
            .args(["start", &service])
            .output()?;

        log::warn!("restored '{service}'");
    }

    Ok(())
}

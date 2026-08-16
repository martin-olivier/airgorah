use crate::globals::*;
use airgorah_common::deps;
use airgorah_common::types::MacMode;
use std::process::Command;

#[derive(thiserror::Error, Debug)]
pub enum IfaceError {
    #[error("Input/Output error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Utf8 conversion error")]
    Utf8Error(#[from] std::string::FromUtf8Error),

    #[error("Interface '{0}' could not be found")]
    IfaceNotFound(String),

    #[error("Could not change MAC address: interface not in monitor mode")]
    IfaceNotMonitor,

    #[error("MAC address is invalid: change its value in the settings page.")]
    InvalidMac,

    #[error("Could not enable monitor mode on '{0}'")]
    MonitorFailed(String),

    #[error("Could not disable monitor mode on '{0}'")]
    ManagedFailed(String),
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

/// Set the MAC address of an interface according to the requested mode.
pub fn set_mac_address(iface: &str, mac: &MacMode) -> Result<(), IfaceError> {
    if !is_monitor_mode(iface)? {
        return Err(IfaceError::IfaceNotMonitor);
    }

    Command::new("ip")
        .args(["link", "set", "dev", iface, "down"])
        .output()?;

    let success = match mac {
        MacMode::Random => {
            Command::new("macchanger").args(["-A", iface]).output()?;
            true
        }
        MacMode::Default => {
            Command::new("macchanger").args(["-p", iface]).output()?;
            true
        }
        MacMode::Specific(mac) => Command::new("macchanger")
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

    log::info!("{iface}: MAC address changed");

    Ok(())
}

/// Switch an interface's 802.11 type with `iw`, taking the link down for the
/// change and bringing it back up afterwards.
fn set_interface_type(iface: &str, mode: &str) -> Result<bool, IfaceError> {
    Command::new("ip")
        .args(["link", "set", "dev", iface, "down"])
        .output()?;

    let set_type_cmd = Command::new("iw")
        .args(["dev", iface, "set", "type", mode])
        .output()?;

    Command::new("ip")
        .args(["link", "set", "dev", iface, "up"])
        .output()?;

    Ok(set_type_cmd.status.success())
}

/// Enable monitor mode on an interface.
pub fn enable_monitor_mode(iface: &str, kill_network_manager: bool) -> Result<String, IfaceError> {
    kill_network_manager_services(kill_network_manager);

    if is_monitor_mode(iface)? {
        *IFACE_WAS_MONITOR.lock().unwrap() = true;
        return Ok(iface.to_string());
    }

    if !set_interface_type(iface, "monitor")? {
        return Err(IfaceError::MonitorFailed(iface.to_string()));
    }

    log::info!("{iface}: monitor mode enabled");

    Ok(iface.to_string())
}

/// Disable monitor mode on an interface, switching it back to managed mode.
pub fn disable_monitor_mode(iface: &str) -> Result<(), IfaceError> {
    if !is_monitor_mode(iface)? {
        return Ok(());
    }

    let mut iface_was_monitor = IFACE_WAS_MONITOR.lock().unwrap();
    if *iface_was_monitor {
        *iface_was_monitor = false;
        return Ok(());
    }
    drop(iface_was_monitor);

    if !set_interface_type(iface, "managed")? {
        return Err(IfaceError::ManagedFailed(iface.to_string()));
    }

    log::info!("{iface}: monitor mode disabled");

    Ok(())
}

/// Get the current interface
pub fn get_iface() -> Option<String> {
    IFACE.lock().unwrap().clone()
}

/// Set the current interface
pub fn set_iface(iface: String) {
    IFACE.lock().unwrap().replace(iface);
}

/// Clear the current interface
pub fn clear_iface() {
    IFACE.lock().unwrap().take();
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

/// Kill the network managers to avoid channel hopping conflicts.
///
/// `enabled` reflects the GUI's `kill_network_manager` setting. If `systemctl` is
/// absent the request is skipped rather than failed (the GUI already disables the
/// option in that case).
fn kill_network_manager_services(enabled: bool) {
    if !enabled {
        return;
    }

    if !deps::is_installed(deps::SYSTEMCTL) {
        log::warn!("systemctl not found, skipping network manager kill");
        return;
    }

    for service in INTERFERENCE_SERVICES {
        let is_service_running = match Command::new("systemctl")
            .args(["is-active", service])
            .output()
        {
            Ok(out) => out.status.success(),
            Err(_) => continue,
        };

        if is_service_running {
            Command::new("systemctl")
                .args(["stop", service])
                .output()
                .ok();

            SERVICES_TO_RESTORE
                .lock()
                .unwrap()
                .push(service.to_string());

            log::warn!("killed '{service}'");
        }
    }
}

/// Restore any network manager services that were killed. A no-op when nothing
/// was stopped.
pub fn restore_network_manager() -> Result<(), IfaceError> {
    if !deps::is_installed(deps::SYSTEMCTL) {
        return Ok(());
    }

    let services_to_restore: Vec<_> = SERVICES_TO_RESTORE.lock().unwrap().drain(..).collect();

    for service in services_to_restore {
        Command::new("systemctl")
            .args(["start", &service])
            .output()?;

        log::warn!("restored '{service}'");
    }

    Ok(())
}

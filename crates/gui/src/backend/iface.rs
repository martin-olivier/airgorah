//! Unprivileged interface queries, run directly by the GUI.
//!
//! Listing interfaces (`iw dev`) and probing 5 GHz capability (sysfs + `iw phy`)
//! need no privilege, so they stay in the GUI. Keeping them here — rather than
//! behind the agent — is what lets the interface picker open and populate
//! without ever escalating; the agent is only started once the user commits to
//! a privileged action (putting a card into monitor mode).

use super::AgentError;
use std::process::Command;

/// Get the available wireless interfaces.
pub fn get_interfaces() -> Result<Vec<String>, AgentError> {
    let cmd = Command::new("sh")
        .args(["-c", "iw dev | awk '$1==\"Interface\"{print $2}'"])
        .output()
        .map_err(|e| AgentError(format!("failed to list interfaces: {e}")))?;

    if !cmd.status.success() {
        return Err(AgentError("failed to retrieve interfaces list".to_string()));
    }

    let out = String::from_utf8_lossy(&cmd.stdout);

    Ok(out.split_terminator('\n').map(String::from).collect())
}

/// Check if an interface supports 5 GHz.
pub fn is_5ghz_supported(iface: &str) -> Result<bool, AgentError> {
    let phy_path = format!("/sys/class/net/{iface}/phy80211");

    let phy_link = std::fs::read_link(&phy_path)
        .map_err(|e| AgentError(format!("could not read '{phy_path}': {e}")))?;

    let phy_name = phy_link
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| AgentError("could not parse PHY name".to_string()))?;

    let check_band_cmd = Command::new("iw")
        .args(["phy", phy_name, "info"])
        .output()
        .map_err(|e| AgentError(format!("failed to query PHY '{phy_name}': {e}")))?;

    if !check_band_cmd.status.success() {
        return Err(AgentError(format!("PHY '{phy_name}' could not be found")));
    }

    let output = String::from_utf8_lossy(&check_band_cmd.stdout);

    Ok(output.contains("5200 MHz") || output.contains("5200.0 MHz"))
}

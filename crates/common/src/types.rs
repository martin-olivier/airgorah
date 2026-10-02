//! Wire types shared across the IPC boundary.
//!
//! Everything here derives `Serialize`/`Deserialize` so it can travel over the
//! agent socket. Types that hold live process handles (e.g. the agent's
//! `Child` processes) deliberately do *not* live here — they stay internal to
//! the agent and are projected onto the serializable [`AttackState`] for the
//! wire.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// How the MAC address of an interface should be set when entering monitor mode.
///
/// Resolved GUI-side from the user's [`Settings::mac_address`] and passed to the
/// agent as an explicit request parameter, so the agent never has to read the
/// user's configuration.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MacMode {
    /// Randomize the MAC address (`macchanger -A`).
    Random,
    /// Restore the permanent hardware MAC (`macchanger -p`).
    Default,
    /// Set a specific MAC address (`macchanger -m <mac>`).
    Specific(String),
}

/// Serializable view of what kind of attack an AP is currently under.
///
/// The agent keeps the actual thread handles internally; this is the shape the
/// GUI receives so it can paint the affected rows and drive its controls.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AttackTarget {
    /// A broadcast deauth against every client (`FF:FF:FF:FF:FF:FF`).
    All,
    /// A deauth targeting the listed client MAC addresses.
    Selection(Vec<String>),
    /// A clientless PMKID solicitation: the agent associates with the AP itself
    /// to make it emit an EAPOL message 1 carrying its PMKID.
    Pmkid,
}

/// A single ongoing deauth attack, as reported to the GUI.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AttackState {
    pub ap: AP,
    pub target: AttackTarget,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AP {
    pub essid: String,
    pub bssid: String,
    pub band: String,
    pub channel: String,
    pub power: String,
    pub privacy: String,
    pub hidden: bool,
    pub handshake: bool,
    pub pmkid: bool,
    /// Path of a capture file the *GUI* saved this AP's crackable material to. This
    /// is GUI-side overlay state: the agent always leaves it `None` and the GUI
    /// fills it in from its own bookkeeping before display.
    pub saved_handshake: Option<String>,
    pub first_time_seen: String,
    pub last_time_seen: String,
    pub clients: HashMap<String, Client>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Client {
    pub mac: String,
    pub packets: String,
    pub power: String,
    pub first_time_seen: String,
    pub last_time_seen: String,
    pub vendor: String,
    pub probes: String,
}

/// Which theme variant the GUI asks GTK for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Theme {
    /// Follow the desktop/GTK default (do not override it).
    #[default]
    System,
    /// Force the light variant.
    Light,
    /// Force the dark variant.
    Dark,
}

/// Which columns of the access-point list are shown.
///
/// ESSID is the primary identifier and is always visible, so it has no toggle.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ColumnVisibility {
    pub bssid: bool,
    pub band: bool,
    pub channel: bool,
    pub power: bool,
    pub encryption: bool,
    pub clients: bool,
    pub first_time_seen: bool,
    pub last_time_seen: bool,
    pub handshake: bool,
    pub pmkid: bool,
}

impl Default for ColumnVisibility {
    fn default() -> Self {
        Self {
            bssid: true,
            band: true,
            channel: true,
            power: true,
            encryption: true,
            clients: true,
            first_time_seen: true,
            last_time_seen: true,
            handshake: true,
            pmkid: true,
        }
    }
}

/// Persistent user settings.
///
/// `#[serde(default)]` lets a config file written by an older version (missing the
/// newer fields) load cleanly: any absent field falls back to [`Settings::default`].
/// `columns` is kept last because it serializes as a TOML table, which must follow
/// every scalar field.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub mac_address: String,
    pub display_hidden_ap: bool,
    pub kill_network_manager: bool,
    /// Dwell time per channel while hopping, in milliseconds.
    pub hop_interval: u64,
    /// How often the GUI refreshes its view from the agent, in milliseconds.
    pub refresh_rate: u64,
    /// Default wordlist pre-filled in the WPA decryption window (empty = none).
    pub wordlist_path: String,
    /// Default directory the save/export dialogs open in (empty = GTK default).
    pub save_path: String,
    /// Theme variant requested from GTK.
    pub theme: Theme,
    /// Fire a desktop notification when a handshake or PMKID is captured.
    pub capture_notifications: bool,
    /// Which access-point columns are shown.
    pub columns: ColumnVisibility,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            mac_address: "random".to_string(),
            display_hidden_ap: true,
            kill_network_manager: true,
            hop_interval: 250,
            refresh_rate: 100,
            wordlist_path: String::new(),
            save_path: String::new(),
            theme: Theme::System,
            capture_notifications: true,
            columns: ColumnVisibility::default(),
        }
    }
}

impl Settings {
    /// Resolve the configured MAC preference into a wire [`MacMode`].
    pub fn mac_mode(&self) -> MacMode {
        match self.mac_address.as_str() {
            "random" => MacMode::Random,
            "default" => MacMode::Default,
            mac => MacMode::Specific(mac.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A config file written before the new fields existed must still load, with every
    // new field falling back to its default (guaranteed by `#[serde(default)]`).
    #[test]
    fn loads_legacy_config_with_defaults() {
        let legacy = r#"
mac_address = "default"
display_hidden_ap = false
kill_network_manager = false
"#;
        let s: Settings = toml::from_str(legacy).unwrap();

        assert_eq!(s.mac_address, "default");
        assert!(!s.display_hidden_ap);
        assert!(!s.kill_network_manager);
        assert_eq!(s.hop_interval, 250);
        assert_eq!(s.refresh_rate, 100);
        assert_eq!(s.wordlist_path, "");
        assert_eq!(s.save_path, "");
        assert_eq!(s.theme, Theme::System);
        assert!(s.capture_notifications);
        assert!(s.columns.bssid && s.columns.pmkid);
    }

    // A partial `[columns]` table fills only the named columns; the rest default true.
    #[test]
    fn partial_columns_table_defaults_missing() {
        let cfg = r#"
mac_address = "random"
display_hidden_ap = true
kill_network_manager = true

[columns]
power = false
"#;
        let s: Settings = toml::from_str(cfg).unwrap();

        assert!(!s.columns.power);
        assert!(s.columns.bssid);
        assert!(s.columns.handshake);
    }

    // A full round-trip must preserve every field. `columns` serializes as a table, so
    // this also guards against it being emitted before the scalar fields.
    #[test]
    fn round_trip_preserves_values() {
        let original = Settings {
            mac_address: "00:11:22:33:44:55".to_string(),
            hop_interval: 500,
            refresh_rate: 1000,
            wordlist_path: "/tmp/rockyou.txt".to_string(),
            save_path: "/tmp/caps".to_string(),
            theme: Theme::Dark,
            capture_notifications: false,
            columns: ColumnVisibility {
                channel: false,
                ..ColumnVisibility::default()
            },
            ..Settings::default()
        };

        let text = toml::to_string(&original).unwrap();
        let parsed: Settings = toml::from_str(&text).unwrap();

        assert_eq!(parsed.mac_address, original.mac_address);
        assert_eq!(parsed.hop_interval, 500);
        assert_eq!(parsed.refresh_rate, 1000);
        assert_eq!(parsed.wordlist_path, "/tmp/rockyou.txt");
        assert_eq!(parsed.save_path, "/tmp/caps");
        assert_eq!(parsed.theme, Theme::Dark);
        assert!(!parsed.capture_notifications);
        assert!(!parsed.columns.channel);
        assert!(parsed.columns.bssid);
    }
}

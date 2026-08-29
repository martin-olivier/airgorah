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

/// Serializable view of which clients of an AP are currently under attack.
///
/// The agent keeps the actual `Child` handles internally; this is the shape the
/// GUI receives so it can paint the affected rows.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AttackTarget {
    /// A broadcast deauth against every client (`FF:FF:FF:FF:FF:FF`).
    All,
    /// A deauth targeting the listed client MAC addresses.
    Selection(Vec<String>),
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub mac_address: String,
    pub display_hidden_ap: bool,
    pub kill_network_manager: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            mac_address: "random".to_string(),
            display_hidden_ap: true,
            kill_network_manager: true,
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

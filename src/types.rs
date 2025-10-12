use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Child;

pub enum AttackedClients {
    All(Child),
    Selection(Vec<(String, Child)>),
}

#[derive(PartialEq)]
pub enum AttackSoftware {
    Aireplay,
    Mdk4,
}

pub struct BruteforceCharsetParams {
    pub lowercase: bool,
    pub uppercase: bool,
    pub numbers: bool,
    pub symbols: bool,
}

pub enum BruteforceCharset {
    Params(BruteforceCharsetParams),
    Specific(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AP {
    pub essid: String,
    pub bssid: String,
    pub band: String,
    pub channel: String,
    pub speed: String,
    pub power: String,
    pub privacy: String,
    pub hidden: bool,
    pub handshake: bool,
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

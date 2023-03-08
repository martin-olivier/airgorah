use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Child;

pub enum AttackedClients {
    All(Child),
    Selection(Vec<(String, Child)>),
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
    pub handshake: bool,
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
}

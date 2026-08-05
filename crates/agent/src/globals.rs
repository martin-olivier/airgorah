//! Privileged state owned by the agent.
//!
//! This is the half of the old monolithic `globals.rs` that manipulates or holds
//! handles to privileged resources: the running `airodump` scan, the running
//! `aireplay`/`mdk4` attacks, the accumulated scan data, and the interface/service
//! state that must be restored on teardown.

use airgorah_common::types::{AP, Client};

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::process::Child;
use std::sync::Mutex;

/// Root-owned 0700 directory for the agent's scan/capture files.
pub static CAPTURE_DIR: &str = "/var/lib/airgorah";

pub static LIVE_SCAN_PATH: &str = "/var/lib/airgorah/live_scan";
pub static OLD_SCAN_PATH: &str = "/var/lib/airgorah/old_scan";
pub static MERGE_SCAN_PATH: &str = "/var/lib/airgorah/merge_scan";

/// Live process handles for an ongoing deauth attack. Kept agent-internal (it
/// holds `Child`s) and projected onto [`airgorah_common::types::AttackState`] for
/// the wire.
pub enum AttackedClients {
    All(Child),
    Selection(Vec<(String, Child)>),
}

pub type AttackPool = HashMap<String, (AP, AttackedClients)>;

lazy_static! {
    pub static ref IFACE: Mutex<Option<String>> = Mutex::new(None);
    pub static ref IFACE_WAS_MONITOR: Mutex<bool> = Mutex::new(false);
    pub static ref SCAN_PROC: Mutex<Option<Child>> = Mutex::new(None);
    pub static ref APS: Mutex<HashMap<String, AP>> = Mutex::new(HashMap::new());
    pub static ref UNLINKED_CLIENTS: Mutex<HashMap<String, Client>> = Mutex::new(HashMap::new());
    pub static ref ATTACK_POOL: Mutex<AttackPool> = Mutex::new(HashMap::new());
    pub static ref SERVICES_TO_RESTORE: Mutex<Vec<String>> = Mutex::new(vec![]);
    pub static ref VENDORS_CACHE: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

//! Privileged state owned by the agent.
//!
//! This is the half of the old monolithic `globals.rs` that manipulates or holds
//! handles to privileged resources: the running native scan, the running native
//! deauth attacks, the accumulated scan data, and the interface/service state that
//! must be restored on teardown.

use airgorah_common::types::{AP, AttackTarget, Client};

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU32;
use std::thread::JoinHandle;

/// Root-owned 0700 directory for the agent's scan/capture files.
pub static CAPTURE_DIR: &str = "/var/lib/airgorah";

pub static LIVE_SCAN_PATH: &str = "/var/lib/airgorah/live_scan";
pub static OLD_SCAN_PATH: &str = "/var/lib/airgorah/old_scan";

/// Handle to the running native capture thread.
///
/// `channels` is the live channel plan the thread re-reads each loop, so swapping
/// it retunes a running scan without restarting the thread; `iface` identifies the
/// interface the scan runs on, so a request for the same one can adapt it in place.
/// `channel` is the channel the thread is currently tuned to (0 until the first
/// hop), published so the GUI can show it. `stop` is raised to ask the thread to
/// exit; `handle` is joined to wait for it to finish flushing the capture file.
pub struct ScanHandle {
    pub iface: String,
    pub channels: Arc<Mutex<Vec<u32>>>,
    pub channel: Arc<AtomicU32>,
    pub stop: Arc<AtomicBool>,
    pub handle: JoinHandle<()>,
}

/// A running native deauth attack against one AP. `target` is kept for the wire
/// projection ([`airgorah_common::types::AttackState`]), `stop` asks the injection
/// thread to exit and `handle` joins it.
pub struct DeauthAttack {
    pub ap: AP,
    pub target: AttackTarget,
    pub stop: Arc<AtomicBool>,
    pub handle: JoinHandle<()>,
}

pub type AttackPool = HashMap<String, DeauthAttack>;

lazy_static! {
    pub static ref IFACE: Mutex<Option<String>> = Mutex::new(None);
    pub static ref IFACE_WAS_MONITOR: Mutex<bool> = Mutex::new(false);
    pub static ref SCAN_HANDLE: Mutex<Option<ScanHandle>> = Mutex::new(None);
    pub static ref APS: Mutex<HashMap<String, AP>> = Mutex::new(HashMap::new());
    pub static ref UNLINKED_CLIENTS: Mutex<HashMap<String, Client>> = Mutex::new(HashMap::new());
    pub static ref ATTACK_POOL: Mutex<AttackPool> = Mutex::new(HashMap::new());
    pub static ref SERVICES_TO_RESTORE: Mutex<Vec<String>> = Mutex::new(vec![]);
    pub static ref VENDORS_CACHE: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

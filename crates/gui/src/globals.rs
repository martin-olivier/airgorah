use airgorah_common::types::{AP, AttackState, Client, Settings};

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::thread::JoinHandle;

pub static APP_ID: &str = "com.molivier.airgorah";
pub use airgorah_common::VERSION;

pub static APP_ICON: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/icons/app_icon.png"));
pub static DEAUTH_ICON: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/icons/deauth.png"));
pub static STOP_ICON: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/icons/stop.png"));
pub static CAPTURE_ICON: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/icons/capture.png"));

lazy_static! {
    pub static ref IFACE: Mutex<Option<String>> = Mutex::new(None);
    pub static ref UPDATE_PROC: Mutex<Option<JoinHandle<bool>>> = Mutex::new(None);

    // Local mirror of the agent's scan data, refreshed from each `GetScanData` snapshot.
    pub static ref APS: Mutex<HashMap<String, AP>> = Mutex::new(HashMap::new());
    pub static ref UNLINKED_CLIENTS: Mutex<HashMap<String, Client>> = Mutex::new(HashMap::new());
    pub static ref ATTACK_POOL: Mutex<HashMap<String, AttackState>> = Mutex::new(HashMap::new());

    /// Channel the interface is currently listening on, mirrored from each
    /// `GetScanData` snapshot, `None` when no scan is running.
    pub static ref CURRENT_CHANNEL: Mutex<Option<u32>> = Mutex::new(None);

    /// GUI-side overlay: which APs have had their handshake saved to which file.
    /// Merged onto incoming snapshots for display (the agent knows nothing of it).
    pub static ref SAVED_HANDSHAKES: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());

    pub static ref SETTINGS: Mutex<Settings> = Mutex::new(Settings::default());
    pub static ref NEW_VERSION: Mutex<Option<String>> = Mutex::new(None);
}

/// Whether the channel controls are currently locked because at least one deauth attack is running.
pub static CHANNEL_LOCK_ACTIVE: AtomicBool = AtomicBool::new(false);

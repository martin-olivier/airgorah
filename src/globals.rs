use crate::types::*;

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::process::Child;
use std::sync::Mutex;
use std::thread::JoinHandle;

pub static APP_ID: &str = "com.molivier.airgorah";
pub static VERSION: &str = "v0.7.2";

pub static LIVE_SCAN_PATH: &str = "/tmp/airgorah_live_scan";
pub static OLD_SCAN_PATH: &str = "/tmp/airgorah_old_scan";
pub static MERGE_SCAN_PATH: &str = "/tmp/airgorah_merge_scan";

pub static CONFIG_PATH: &str = "/etc/airgorah/config.toml";

pub static APP_ICON: &[u8] = include_bytes!("../icons/app_icon.png");
pub static DEAUTH_ICON: &[u8] = include_bytes!("../icons/deauth.png");
pub static STOP_ICON: &[u8] = include_bytes!("../icons/stop.png");
pub static CAPTURE_ICON: &[u8] = include_bytes!("../icons/capture.png");

pub type AttackPool = HashMap<String, (AP, AttackedClients)>;

lazy_static! {
    pub static ref IFACE: Mutex<Option<String>> = Mutex::new(None);
    pub static ref UPDATE_PROC: Mutex<Option<JoinHandle<bool>>> = Mutex::new(None);
    pub static ref SCAN_PROC: Mutex<Option<Child>> = Mutex::new(None);
    pub static ref APS: Mutex<HashMap<String, AP>> = Mutex::new(HashMap::new());
    pub static ref UNLINKED_CLIENTS: Mutex<HashMap<String, Client>> = Mutex::new(HashMap::new());
    pub static ref ATTACK_POOL: Mutex<AttackPool> = Mutex::new(HashMap::new());
    pub static ref VENDORS_CACHE: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
    pub static ref SETTINGS: Mutex<Settings> = Mutex::new(Settings::default());
    pub static ref NEW_VERSION: Mutex<Option<String>> = Mutex::new(None);
    pub static ref SERVICES_TO_RESTORE: Mutex<Vec<String>> = Mutex::new(vec![]);
}

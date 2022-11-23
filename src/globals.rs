use crate::types::*;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::process::Child;
use std::sync::Mutex;
use std::thread::JoinHandle;

pub static APP_ID: &str = "com.martin-olivier.airgorah";
pub static VERSION: &str = "v0.2.0";
pub static SCAN_PATH: &str = "/tmp/airgorah_scan";
pub static CAPTURE_PATH: &str = "/tmp/airgorah_capture";

pub static APP_ICON: &[u8] = include_bytes!("../icons/app_icon.png");
pub static DEAUTH_ICON: &[u8] = include_bytes!("../icons/deauth.png");
pub static STOP_ICON: &[u8] = include_bytes!("../icons/stop.png");
pub static CAPTURE_ICON: &[u8] = include_bytes!("../icons/capture.png");

lazy_static! {
    pub static ref IFACE: Mutex<Option<String>> = Mutex::new(None);
    pub static ref UPDATE_PROC: Mutex<Option<JoinHandle<bool>>> = Mutex::new(None);
    pub static ref SCAN_PROC: Mutex<Option<Child>> = Mutex::new(None);
    pub static ref CAPTURE_PROC: Mutex<Option<Child>> = Mutex::new(None);
    pub static ref APS: Mutex<HashMap<String, AP>> = Mutex::new(HashMap::new());
    pub static ref ATTACK_POOL: Mutex<HashMap<String, (AP, AttackedClients)>> =
        Mutex::new(HashMap::new());
}

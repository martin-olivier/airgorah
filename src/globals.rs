use crate::types::*;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::process::Child;
use std::sync::Mutex;

pub static APP_ID: &str = "com.martin-olivier.airgorah";
pub static VERSION: &str = "v0.1.0";
pub static SCAN_PATH: &str = "/tmp/airgorah";

pub static APP_ICON: &[u8] = include_bytes!("../icons/app_icon.png");

lazy_static! {
    pub static ref IFACE: Mutex<Option<String>> = Mutex::new(None);
    pub static ref SCAN_PROC: Mutex<Option<Child>> = Mutex::new(None);
    pub static ref APS: Mutex<HashMap<String, AP>> = Mutex::new(HashMap::new());
    pub static ref ATTACK_POOL: Mutex<HashMap<String, (AP, AttackedClients)>> =
        Mutex::new(HashMap::new());
}

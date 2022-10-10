use lazy_static::lazy_static;
use std::process::Child;
use std::sync::Mutex;
use crate::backend::AP;

pub static SCAN_PATH: &'static str = "/tmp/airgorah";

lazy_static! {
    pub static ref IFACE: Mutex<Option<String>> = Mutex::new(None);
    pub static ref SCAN_PROC: Mutex<Option<Child>> = Mutex::new(None);
    pub static ref SELECTED_AP: Mutex<Option<AP>> = Mutex::new(None);
}

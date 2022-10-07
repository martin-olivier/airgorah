use lazy_static::lazy_static;
use std::process::Child;
use std::sync::Mutex;

pub static SCAN_PATH: &'static str = "/tmp/airgorah";

lazy_static! {
    pub(crate) static ref IFACE: Mutex<Option<String>> = Mutex::new(None);
    pub(crate) static ref SCAN_PROC: Mutex<Option<Child>> = Mutex::new(None);
}

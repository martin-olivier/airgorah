use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    pub(crate) static ref IFACE: Mutex<String> = Mutex::new("".to_string());
}

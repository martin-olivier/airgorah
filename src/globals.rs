use lazy_static::lazy_static;
use std::collections::HashMap;
use std::process::Child;
use std::sync::Mutex;
use crate::backend::AP;

pub static SCAN_PATH: &'static str = "/tmp/airgorah";

pub enum AttackTargets {
    All(Child),
    Selection(Vec<(String, Child)>),
}

lazy_static! {
    pub static ref IFACE: Mutex<Option<String>> = Mutex::new(None);
    pub static ref SCAN_PROC: Mutex<Option<Child>> = Mutex::new(None);
    pub static ref APS: Mutex<Vec<AP>> = Mutex::new(Vec::<AP>::new());
    pub static ref ATTACK_POOL: Mutex<HashMap<String, AttackTargets>> = Mutex::new(HashMap::new());
}

use crate::error::Error;
use crate::globals::*;
use crate::types::*;

pub fn check_dependencies(deps: &[&str]) -> Result<(), Error> {
    for dep in deps {
        if which::which(dep).is_err() {
            return Err(Error::new(&format!(
                "Missing dependency: \"{}\"\n{}",
                dep, "Please install it using your package manager"
            )));
        }
    }
    Ok(())
}

pub fn app_cleanup() {
    if let Some(child) = SCAN_PROC.lock().unwrap().as_mut() {
        child.kill().unwrap();
        child.wait().unwrap();
    }

    for attacked_ap in ATTACK_POOL.lock().unwrap().iter_mut() {
        match &mut attacked_ap.1 .1 {
            AttackedClients::All(child) => {
                child.kill().unwrap();
                child.wait().unwrap();
            }
            AttackedClients::Selection(child_list) => {
                for (_cli, child) in child_list {
                    child.kill().unwrap();
                    child.wait().unwrap();
                }
            }
        }
    }

    if let Some(iface) = IFACE.lock().unwrap().as_ref() {
        super::disable_monitor_mode(iface).ok();
        super::restore_network_manager().ok();
    }

    std::fs::remove_file(SCAN_PATH.to_string() + "-01.csv").ok();
}

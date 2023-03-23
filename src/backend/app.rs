use crate::error::Error;
use crate::globals::*;
use crate::types::*;

use std::thread::JoinHandle;

pub fn app_setup() -> Result<(), Error> {
    app_cleanup();

    ctrlc::set_handler(move || {
        app_cleanup();
        std::process::exit(1);
    })
    .expect("Error setting Ctrl-C handler");

    if sudo::check() != sudo::RunningAs::Root {
        return Err(Error::new("Airgorah need root privilege to run"));
    }

    check_dependencies(&[
        "sh",
        "service",
        "iw",
        "iwlist",
        "awk",
        "airmon-ng",
        "airodump-ng",
        "aireplay-ng",
        "aircrack-ng",
        "gnome-terminal",
        "mergecap",
    ])
}

pub fn app_cleanup() {
    super::stop_scan_process();

    for attacked_ap in super::get_attack_pool().iter_mut() {
        match &mut attacked_ap.1 .1 {
            AttackedClients::All(child) => {
                child.kill().unwrap();
                child.wait().unwrap();
            }
            AttackedClients::Selection(child_list) => {
                for (_, child) in child_list {
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

    std::fs::remove_file(LIVE_SCAN_PATH.to_string() + "-01.csv").ok();
    std::fs::remove_file(LIVE_SCAN_PATH.to_string() + "-01.cap").ok();
    std::fs::remove_file(OLD_SCAN_PATH.to_string() + "-01.cap").ok();
}

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

pub fn spawn_update_checker() -> JoinHandle<bool> {
    std::thread::spawn(|| {
        let url = "https://api.github.com/repos/martin-olivier/airgorah/releases/latest";

        if let Ok(response) = ureq::get(url).call() {
            if let Ok(json) = response.into_json::<serde_json::Value>() {
                return json["tag_name"] != VERSION;
            }
        }
        false
    })
}

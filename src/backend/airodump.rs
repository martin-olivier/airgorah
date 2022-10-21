use std::sync::MutexGuard;

use crate::globals::*;
use crate::types::*;
use crate::error::Error;

use serde::Deserialize;

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

pub fn set_scan_process(args: &[&str]) -> Result<(), Error> {
    let iface = match super::get_iface().as_ref() {
        Some(res) => res.to_string(),
        None => return Err(Error::new("No interface set")),
    };

    if let Some(child) = SCAN_PROC.lock().unwrap().as_mut() {
        child.kill().unwrap();
        child.wait().unwrap();
    }

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
        .spawn()?;

    SCAN_PROC.lock().unwrap().replace(child);

    Ok(())
}

pub fn stop_scan_process() {
    if let Some(child) = SCAN_PROC.lock().unwrap().as_mut() {
        child.kill().unwrap();
        child.wait().unwrap();
    }

    SCAN_PROC.lock().unwrap().take();
}

pub fn get_airodump_data() -> Option<Vec<AP>> {
    let mut aps = vec![];
    for attacked_ap in super::get_attack_pool().iter() {
        aps.push(attacked_ap.1.0.clone());
    }

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
        if let Ok(res) = result {
            let channel_nb = res.channel.trim_start().parse::<i32>().unwrap_or(-1);
            let band = if channel_nb > 14 {
                "5 GHz".to_string()
            } else {
                "2.4 GHz".to_string()
            };
            let mut essid = res.essid.trim_start().to_string();

            if essid.is_empty() {
                essid = format!("[Hidden ESSID] (length: {})", res.id_length.trim_start());
            }

            aps.push(AP {
                essid,
                bssid: res.bssid.trim_start().to_string(),
                band,
                channel: res.channel.trim_start().to_string(),
                speed: res.speed.trim_start().to_string(),
                power: res.power.trim_start().to_string(),
                privacy: res.privacy.trim_start().to_string(),
                first_time_seen: res.first_time_seen.trim_start().to_string(),
                last_time_seen: res.last_time_seen.trim_start().to_string(),
                clients: vec![],
            });
        }
    }

    for result in cli_reader.deserialize::<RawClient>() {
        if let Ok(res) = result {
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
    }

    Some(aps)
}

pub fn get_aps() -> MutexGuard<'static, Vec<AP>> {
    APS.lock().unwrap()
}
use regex::Regex;
use std::collections::HashMap;
use std::sync::MutexGuard;

use crate::error::Error;
use crate::globals::*;
use crate::types::*;

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

pub fn is_scan_process() -> bool {
    SCAN_PROC.lock().unwrap().as_ref().is_some()
}

pub fn is_valid_channel_filter(channel_filter: &str) -> bool {
    let channel_regex = Regex::new(r"^[1-9]+[0-9]*$").unwrap();
    let channel_list: Vec<String> = channel_filter
        .split_terminator(',')
        .map(String::from)
        .collect();

    for chan in channel_list {
        if !channel_regex.is_match(&chan) {
            return false;
        }
    }

    true
}

pub fn set_scan_process(args: &[&str]) -> Result<(), Error> {
    let iface = match super::get_iface().as_ref() {
        Some(res) => res.to_string(),
        None => return Err(Error::new("No interface set")),
    };

    stop_scan_process();

    let mut proc_args = vec![
        iface.as_str(),
        "-a",
        "--output-format",
        "csv,cap",
        "-w",
        LIVE_SCAN_PATH,
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
        std::process::Command::new("kill")
            .arg(child.id().to_string())
            .status()
            .unwrap();
        child.wait().ok();
    }

    SCAN_PROC.lock().unwrap().take();

    let old_path_exists = std::path::Path::new(&(OLD_SCAN_PATH.to_string() + "-01.cap")).exists();
    let live_path_exists = std::path::Path::new(&(LIVE_SCAN_PATH.to_string() + "-01.cap")).exists();

    std::fs::remove_file(LIVE_SCAN_PATH.to_string() + "-01.csv").ok();

    if !live_path_exists {
        return;
    }

    if !old_path_exists {
        std::fs::rename(
            LIVE_SCAN_PATH.to_string() + "-01.cap",
            OLD_SCAN_PATH.to_string() + "-01.cap",
        )
        .ok();
        return;
    }

    std::process::Command::new("mergecap")
        .args([
            "-a",
            "-F",
            "pcap",
            "-w",
            &(MERGE_SCAN_PATH.to_string() + "-01.cap"),
            &(OLD_SCAN_PATH.to_string() + "-01.cap"),
            &(LIVE_SCAN_PATH.to_string() + "-01.cap"),
        ])
        .status()
        .unwrap();

    std::fs::remove_file(LIVE_SCAN_PATH.to_string() + "-01.cap").ok();
    std::fs::remove_file(OLD_SCAN_PATH.to_string() + "-01.cap").ok();
    std::fs::rename(
        MERGE_SCAN_PATH.to_string() + "-01.cap",
        OLD_SCAN_PATH.to_string() + "-01.cap",
    )
    .ok();
}

pub fn get_airodump_data() -> HashMap<String, AP> {
    let mut aps: HashMap<String, AP> = HashMap::new();

    for attacked_ap in super::get_attack_pool().iter() {
        aps.insert(attacked_ap.0.clone(), attacked_ap.1 .0.clone());
    }

    let mut glob_aps = super::get_aps();

    for ap in glob_aps.iter() {
        aps.insert(ap.0.clone(), ap.1.clone());
    }

    let full_path = LIVE_SCAN_PATH.to_string() + "-01.csv";
    let csv_file = match std::fs::read_to_string(full_path) {
        Ok(file) => file,
        Err(_) => return aps,
    };

    let file_parts: Vec<&str> = csv_file.split("\r\n\r\n").collect();
    let ap_part = if !file_parts.is_empty() {
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

    for result in ap_reader.deserialize::<RawAP>().flatten() {
        let channel_nb = result.channel.trim_start().parse::<i32>().unwrap_or(-1);
        let band = if channel_nb > 14 {
            "5 GHz".to_string()
        } else {
            "2.4 GHz".to_string()
        };

        let bssid = result.bssid.trim_start().to_string();

        let mut essid = result.essid.trim_start().to_string();

        if essid.is_empty() {
            essid = format!("[Hidden] (length: {})", result.id_length.trim_start());
        }

        let old_data = aps.insert(
            bssid.clone(),
            AP {
                essid,
                bssid: bssid.clone(),
                band,
                channel: result.channel.trim_start().to_string(),
                speed: result.speed.trim_start().to_string(),
                power: result.power.trim_start().to_string(),
                privacy: result.privacy.trim_start().to_string(),
                handshake: {
                    match glob_aps.get(&bssid) {
                        Some(ap) => ap.handshake,
                        None => false,
                    }
                },
                saved_handshake: match glob_aps.get(&bssid) {
                    Some(ap) => ap.saved_handshake.clone(),
                    None => None,
                },
                first_time_seen: result.first_time_seen.trim_start().to_string(),
                last_time_seen: result.last_time_seen.trim_start().to_string(),
                clients: HashMap::new(),
            },
        );

        if let Some(ap) = old_data {
            aps.get_mut(&bssid).unwrap().clients = ap.clients;
        }
    }

    for result in cli_reader.deserialize::<RawClient>().flatten() {
        if let Some(ap) = aps.get_mut(result.bssid.trim_start()) {
            let mac = result.station_mac.trim_start().to_string();
            ap.clients.insert(
                mac.clone(),
                Client {
                    mac,
                    packets: result.packets.trim_start().to_string(),
                    power: result.power.trim_start().to_string(),
                    first_time_seen: result.first_time_seen.trim_start().to_string(),
                    last_time_seen: result.last_time_seen.trim_start().to_string(),
                },
            );
        }
    }

    for ap in aps.iter() {
        glob_aps.insert(ap.0.clone(), ap.1.clone());
    }

    aps
}

pub fn get_aps() -> MutexGuard<'static, HashMap<String, AP>> {
    APS.lock().unwrap()
}

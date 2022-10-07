use crate::globals::{IFACE, SCAN_PATH, SCAN_PROC};
use csv::{Error, Position, ReaderBuilder};
use network_interface::NetworkInterfaceConfig;
use std::process::Command;

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
    cipher: String,
    #[serde(rename = " Authentication")]
    authentication: String,
    #[serde(rename = " Power")]
    power: String,
    #[serde(rename = " # beacons")]
    beacons: String,
    #[serde(rename = " # IV")]
    iv: String,
    #[serde(rename = " LAN IP")]
    lan_ip: String,
    #[serde(rename = " ID-length")]
    id_length: String,
    #[serde(rename = " ESSID")]
    essid: String,
    #[serde(rename = " Key")]
    key: String,
}

#[derive(Clone)]
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
    probed_essids: String,
}

#[derive(Clone)]
pub struct Client {
    pub mac: String,
    pub packets: String,
    pub power: String,
    pub first_time_seen: String,
    pub last_time_seen: String,
}

pub fn get_interfaces() -> Vec<String> {
    let mut ret = vec![];
    let ifaces = network_interface::NetworkInterface::show().unwrap();

    for iface in ifaces {
        if !ret.contains(&iface.name) && iface.name != "lo" {
            ret.push(iface.name);
        }
    }
    ret
}

pub fn enable_monitor_mode(iface: &str) -> Result<String, ()> {
    let output = Command::new("airmon-ng")
        .args(["start", iface])
        .output()
        .expect("failed to execute process: airmon-ng");

    match output.status.success() {
        true => Ok(iface.to_string() + "mon"),
        false => Err(()),
    }
}

pub fn set_scan_process(args: &[&str]) {
    let iface = match IFACE.lock().unwrap().as_ref() {
        Some(res) => res.to_string(),
        None => return,
    };

    match SCAN_PROC.lock().unwrap().as_mut() {
        Some(child) => child.kill().unwrap(),
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
        .unwrap();
    SCAN_PROC.lock().unwrap().replace(child);
}

pub fn get_airodump_data() -> Option<Vec<AP>> {
    let mut aps = vec![];
    let full_path = SCAN_PATH.to_string() + "-01.csv";
    let csv_file = match std::fs::read_to_string(full_path) {
        Ok(file) => file,
        Err(_) => return None
    };

    let file_parts: Vec<&str> = csv_file.split("\r\n\r\n").collect();
    let ap_part = if file_parts.len() >= 1 {file_parts[0]} else {""};
    let cli_part = if file_parts.len() >= 2 {file_parts[1]} else {""};

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
                            band: if channel_nb > 14 {"5 GHz".to_string()} else {"2.4 GHz".to_string()},
                            channel: res.channel.trim_start().to_string(),
                            speed: res.speed.trim_start().to_string(),
                            power: res.power.trim_start().to_string(),
                            privacy: res.privacy.trim_start().to_string(),
                            first_time_seen: res.first_time_seen.trim_start().to_string(),
                            last_time_seen: res.last_time_seen.trim_start().to_string(),
                            clients: vec![],
                        })
                    }
                    Err(_) => {},
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
                    Err(_) => {},
                }
            }

    Some(aps)
}

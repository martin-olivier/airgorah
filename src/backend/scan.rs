use std::collections::HashMap;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::MutexGuard;

use super::*;
use crate::globals::*;
use crate::types::*;

use serde::Deserialize;

use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;

/// Represents the AP section of the csv file generated by airodump
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

/// Represents the Client section of the csv file generated by airodump
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
    probes: String,
}

#[derive(thiserror::Error, Debug)]
pub enum ScanError {
    #[error("Could not setup scan process: no band selected")]
    NoBandSelected,

    #[error("Input/Output error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Kill error: errno code: {0}")]
    KillError(#[from] nix::errno::Errno),
}

/// Check if a scan is currently running
pub fn is_scan_process() -> bool {
    SCAN_PROC.lock().unwrap().is_some()
}

/// Check if the content of the channel filter is valid
pub fn is_valid_channel_filter(channel_filter: &str, ghz_2_4_but: bool, ghz_5_but: bool) -> bool {
    let channel_list: Vec<String> = channel_filter
        .split_terminator(',')
        .map(String::from)
        .collect();

    let mut channel_buf = vec![];

    if channel_filter.ends_with(',') {
        return false;
    }

    for channel_str in channel_list {
        let channel = match channel_str.parse::<u32>() {
            Ok(chan) => chan,
            Err(_) => return false,
        };

        if channel < 1 || (15..=35).contains(&channel) || channel > 165 {
            return false;
        }

        if (1..=14).contains(&channel) && !ghz_2_4_but {
            return false;
        }

        if (36..=165).contains(&channel) && !ghz_5_but {
            return false;
        }

        if channel_buf.contains(&channel) {
            return false;
        }

        channel_buf.push(channel);
    }

    true
}

/// Set the scan process
pub fn set_scan_process(
    iface: &str,
    ghz_2_4: bool,
    ghz_5: bool,
    channel_filter: Option<String>,
) -> Result<(), ScanError> {
    if !ghz_2_4 && !ghz_5 {
        return Err(ScanError::NoBandSelected);
    }

    stop_scan_process()?;

    let mut proc_args = vec![
        iface,
        "-a",
        "--output-format",
        "csv,cap",
        "-w",
        LIVE_SCAN_PATH,
        "--write-interval",
        "1",
    ];

    let mut band = String::new();

    if ghz_5 {
        band += "a";
    }

    if ghz_2_4 {
        band += "bg";
    }

    proc_args.push("--band");
    proc_args.push(&band);

    let channels;

    if let Some(ref filter) = channel_filter {
        channels = filter;

        proc_args.push("--channel");
        proc_args.push(channels);
    }

    let child = Command::new("airodump-ng")
        .args(proc_args)
        .stdout(Stdio::null())
        .spawn()?;

    SCAN_PROC.lock().unwrap().replace(child);

    log::info!(
        "scan started: 2.4ghz: {}, 5ghz: {}, channel filter: {:?}",
        ghz_2_4,
        ghz_5,
        channel_filter
    );

    Ok(())
}

/// Stop the scan process
pub fn stop_scan_process() -> Result<(), ScanError> {
    if let Some(child) = SCAN_PROC.lock().unwrap().as_mut() {
        let child_pid = Pid::from_raw(child.id() as i32);

        kill(child_pid, Signal::SIGTERM)?;

        log::info!("scan stopped, sent kill SIGTERM to pid {}", child_pid);

        child.wait()?;
    }

    SCAN_PROC.lock().unwrap().take();

    let old_path_exists = Path::new(&(OLD_SCAN_PATH.to_string() + "-01.cap")).exists();
    let live_path_exists = Path::new(&(LIVE_SCAN_PATH.to_string() + "-01.cap")).exists();

    std::fs::remove_file(LIVE_SCAN_PATH.to_string() + "-01.csv").ok();

    if !live_path_exists {
        return Ok(());
    }

    if !old_path_exists {
        std::fs::rename(
            LIVE_SCAN_PATH.to_string() + "-01.cap",
            OLD_SCAN_PATH.to_string() + "-01.cap",
        )
        .ok();
        return Ok(());
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
        .status()?;

    std::fs::remove_file(LIVE_SCAN_PATH.to_string() + "-01.cap").ok();
    std::fs::remove_file(OLD_SCAN_PATH.to_string() + "-01.cap").ok();
    std::fs::rename(
        MERGE_SCAN_PATH.to_string() + "-01.cap",
        OLD_SCAN_PATH.to_string() + "-01.cap",
    )
    .ok();

    Ok(())
}

/// Get the data captured from airodump
pub fn get_airodump_data() -> HashMap<String, AP> {
    let mut aps: HashMap<String, AP> = HashMap::new();

    for attacked_ap in get_attack_pool().iter() {
        aps.insert(attacked_ap.0.clone(), attacked_ap.1 .0.clone());
    }

    let mut glob_aps = get_aps();

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
        let mut hidden = false;

        let old_ap_data = glob_aps.get(&bssid);

        if essid.is_empty() {
            hidden = true;
            essid = format!("[Hidden] (length: {})", result.id_length.trim_start());

            if let Some(old_ap_data) = old_ap_data {
                if !old_ap_data.essid.starts_with("[Hidden] (length:") {
                    essid = old_ap_data.essid.clone();
                }
            }
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
                hidden,
                handshake: {
                    match old_ap_data {
                        Some(ap) => ap.handshake,
                        None => false,
                    }
                },
                saved_handshake: match old_ap_data {
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
        let mac = result.station_mac.trim_start().to_string();
        let client_vendor = super::find_vendor(&mac);

        match aps.get_mut(result.bssid.trim_start()) {
            Some(ap) => {
                ap.clients.insert(
                    mac.clone(),
                    Client {
                        mac,
                        packets: result.packets.trim_start().to_string(),
                        power: result.power.trim_start().to_string(),
                        first_time_seen: result.first_time_seen.trim_start().to_string(),
                        last_time_seen: result.last_time_seen.trim_start().to_string(),
                        vendor: client_vendor,
                        probes: result.probes.trim_start().to_string(),
                    },
                );
            }
            None => {
                get_unlinked_clients().insert(
                    mac.clone(),
                    Client {
                        mac,
                        packets: result.packets.trim_start().to_string(),
                        power: result.power.trim_start().to_string(),
                        first_time_seen: result.first_time_seen.trim_start().to_string(),
                        last_time_seen: result.last_time_seen.trim_start().to_string(),
                        vendor: client_vendor,
                        probes: result.probes.trim_start().to_string(),
                    },
                );
            }
        }
    }

    for (bssid, ap) in aps.iter() {
        glob_aps.insert(bssid.clone(), ap.clone());
    }

    aps
}

/// Get the APs data collected
pub fn get_aps() -> MutexGuard<'static, HashMap<String, AP>> {
    APS.lock().unwrap()
}

/// Get unlinked clients
pub fn get_unlinked_clients() -> MutexGuard<'static, HashMap<String, Client>> {
    UNLINKED_CLIENTS.lock().unwrap()
}

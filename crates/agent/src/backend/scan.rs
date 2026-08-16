use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::MutexGuard;
use std::sync::atomic::{AtomicBool, Ordering};

use super::get_attack_pool;
use super::sniffer;
use crate::globals::*;
use airgorah_common::types::*;

#[derive(thiserror::Error, Debug)]
pub enum ScanError {
    #[error("Could not setup scan process: no band selected")]
    NoBandSelected,

    #[error("Input/Output error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Check if a scan is currently running
pub fn is_scan_process() -> bool {
    SCAN_HANDLE.lock().unwrap().is_some()
}

/// Start (or restart) the native capture thread.
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

    let channels = sniffer::build_channel_list(ghz_2_4, ghz_5, channel_filter.as_deref());

    let stop = Arc::new(AtomicBool::new(false));
    let thread_stop = stop.clone();
    let thread_iface = iface.to_string();
    let handle = std::thread::spawn(move || {
        sniffer::run(thread_iface, channels, thread_stop);
    });

    SCAN_HANDLE
        .lock()
        .unwrap()
        .replace(ScanHandle { stop, handle });

    log::info!(
        "scan started: 2.4ghz: {ghz_2_4}, 5ghz: {ghz_5}, channel filter: {channel_filter:?}"
    );

    Ok(())
}

/// Stop the capture thread and fold its capture into the accumulated one.
pub fn stop_scan_process() -> Result<(), ScanError> {
    let handle = SCAN_HANDLE.lock().unwrap().take();
    if let Some(scan) = handle {
        scan.stop.store(true, Ordering::Relaxed);
        let _ = scan.handle.join();
        log::info!("scan stopped");
    }

    // Merge the just-finished live capture into the accumulated old capture, so a
    // handshake captured across several scan restarts (e.g. the deauth "park on
    // channel" behaviour) ends up in a single file.
    let old_path_exists = Path::new(&(get_old_scan_path() + get_cap_ext())).exists();
    let live_path_exists = Path::new(&(get_live_scan_path() + get_cap_ext())).exists();

    if !live_path_exists {
        return Ok(());
    }

    if !old_path_exists {
        std::fs::rename(
            get_live_scan_path() + get_cap_ext(),
            get_old_scan_path() + get_cap_ext(),
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
            &(get_merge_scan_path() + get_cap_ext()),
            &(get_old_scan_path() + get_cap_ext()),
            &(get_live_scan_path() + get_cap_ext()),
        ])
        .status()?;

    std::fs::remove_file(get_live_scan_path() + get_cap_ext()).ok();
    std::fs::remove_file(get_old_scan_path() + get_cap_ext()).ok();
    std::fs::rename(
        get_merge_scan_path() + get_cap_ext(),
        get_old_scan_path() + get_cap_ext(),
    )
    .ok();

    Ok(())
}

/// Drop the accumulated scan data (used by the GUI's "restart" action).
pub fn reset_scan_data() {
    get_aps().clear();
    get_unlinked_clients().clear();
}

/// Snapshot of the discovered access points for the GUI.
///
/// The capture thread maintains [`APS`] live, so this just clones it, overlaying
/// any AP currently under attack (which must stay visible even if it stopped
/// beaconing).
pub fn get_airodump_data() -> HashMap<String, AP> {
    let mut aps: HashMap<String, AP> = HashMap::new();

    let attacked: Vec<AP> = get_attack_pool()
        .values()
        .map(|attack| attack.ap.clone())
        .collect();
    for ap in attacked {
        aps.insert(ap.bssid.clone(), ap);
    }

    for (bssid, ap) in get_aps().iter() {
        aps.insert(bssid.clone(), ap.clone());
    }

    aps
}

pub fn get_aps() -> MutexGuard<'static, HashMap<String, AP>> {
    APS.lock().unwrap()
}

pub fn get_unlinked_clients() -> MutexGuard<'static, HashMap<String, Client>> {
    UNLINKED_CLIENTS.lock().unwrap()
}

pub fn get_cap_ext() -> &'static str {
    "-01.cap"
}

pub fn get_live_scan_path() -> String {
    format!("{}-{}", LIVE_SCAN_PATH, std::process::id())
}

pub fn get_old_scan_path() -> String {
    format!("{}-{}", OLD_SCAN_PATH, std::process::id())
}

pub fn get_merge_scan_path() -> String {
    format!("{}-{}", MERGE_SCAN_PATH, std::process::id())
}

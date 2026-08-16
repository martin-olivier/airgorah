use super::*;

/// Stop the scan process, kill all the attack processes, disable monitor mode,
/// restore the network managers, and remove the files created by the app.
///
/// Runs agent-side because the agent owns every one of those resources. It is
/// invoked both on a clean shutdown and when the GUI disconnects unexpectedly, so
/// the wireless card is never left in monitor mode with an orphaned scan running.
pub fn app_cleanup() {
    stop_scan_process().ok();
    stop_all_deauth_attacks();

    if let Some(ref iface) = get_iface() {
        disable_monitor_mode(iface).ok();
    }

    restore_network_manager().ok();

    std::fs::remove_file(get_live_scan_path() + get_cap_ext()).ok();
    std::fs::remove_file(get_old_scan_path() + get_cap_ext()).ok();
}

//! IPC client

use crate::globals::*;
use airgorah_common::deps::{self, Requirer};
use airgorah_common::ipc::*;
use airgorah_common::types::{AP, AttackSoftware, AttackState, Client};

use lazy_static::lazy_static;
use nix::unistd::{geteuid, getuid};
use std::collections::HashMap;
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

/// Error surfaced from an agent interaction. Its `Display` carries the agent's
/// own error text, which the frontend shows in its dialogs.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct AgentError(pub String);

/// The live connection to the privileged agent. Holds the socket and keeps the
/// agent child process alive for the lifetime of the GUI.
struct AgentClient {
    stream: UnixStream,
    _child: Child,
}

impl AgentClient {
    fn exchange(&mut self, request: Request) -> Result<Response, AgentError> {
        exchange(&mut self.stream, request)
    }
}

/// Write a request and read the response on a raw stream.
fn exchange(stream: &mut UnixStream, request: Request) -> Result<Response, AgentError> {
    write_msg(stream, &request)
        .map_err(|e| AgentError(format!("failed to send request to agent: {e}")))?;
    read_msg(stream).map_err(|e| AgentError(format!("lost connection to agent: {e}")))
}

lazy_static! {
    static ref CLIENT: Mutex<Option<AgentClient>> = Mutex::new(None);
}

/// Send a request and get the raw response, or an error if the agent is gone.
fn request(request: Request) -> Result<Response, AgentError> {
    let mut guard = CLIENT.lock().unwrap();
    let client = guard
        .as_mut()
        .ok_or_else(|| AgentError("the privileged agent is not connected".to_string()))?;
    client.exchange(request)
}

/// Collapse a response that should just be an acknowledgement.
fn expect_ok(response: Response) -> Result<(), AgentError> {
    match response {
        Response::Ok => Ok(()),
        Response::Error { message } => Err(AgentError(message)),
        _ => Err(AgentError("unexpected response from agent".to_string())),
    }
}

// --------------------------------------------------------------------------
// Lifecycle
// --------------------------------------------------------------------------

/// Format the "missing required tools" error shown to the user. Both the GUI's
/// own startup check and the agent's `Hello` reply feed through here.
fn missing_deps_error<S: AsRef<str>>(missing: &[S]) -> AgentError {
    let list = missing
        .iter()
        .map(AsRef::as_ref)
        .collect::<Vec<_>>()
        .join(", ");
    AgentError(format!("Missing required dependencies: {list}"))
}

/// Startup initialization that needs no privilege: load settings and warn if the
/// GUI is (pointlessly) running as root. The privileged agent is intentionally
/// *not* started here — see [`ensure_agent`] — but we verify it is installed so a
/// broken install fails clearly at launch rather than only on the first scan.
/// Locating the binary does not run it and does not escalate.
pub fn init() -> Result<(), AgentError> {
    if geteuid().is_root() {
        log::warn!(
            "running the airgorah GUI as root is discouraged and does not work under Wayland"
        );
    }

    super::load_settings();

    agent_binary_path()?;

    let missing = deps::missing_required(Requirer::Gui);
    if !missing.is_empty() {
        return Err(missing_deps_error(&missing));
    }

    Ok(())
}

/// Ensure the privileged agent is running and connected, launching it (via
/// pkexec) on first use and performing the version/dependency handshake.
///
/// This is the single escalation point, it is called only when the user commits
/// to a privileged action.
pub fn ensure_agent() -> Result<(), AgentError> {
    if CLIENT.lock().unwrap().is_some() {
        return Ok(());
    }

    let uid = getuid().as_raw();
    // Key the socket on our pid so multiple GUI instances don't collide. The
    // agent derives the same id from its parent pid (it is exec'd by pkexec, or
    // spawned directly, so its parent is this GUI), so we need not pass it.
    let sock = socket_path(uid, std::process::id());

    // Spawn + connect without holding the CLIENT lock: this may block on the
    // polkit prompt. Only the GTK main thread calls this, so there is no race.
    let mut child = spawn_agent()?;

    match connect_agent(&sock, &mut child) {
        Ok(stream) => {
            *CLIENT.lock().unwrap() = Some(AgentClient {
                stream,
                _child: child,
            });
            Ok(())
        }
        Err(e) => {
            child.kill().ok();
            child.wait().ok();
            Err(e)
        }
    }
}

/// Connect to the freshly spawned agent and run the version/dependency check.
fn connect_agent(sock: &str, child: &mut Child) -> Result<UnixStream, AgentError> {
    let mut stream = connect_with_timeout(sock, child)?;

    match exchange(
        &mut stream,
        Request::Hello {
            version: VERSION.to_string(),
        },
    )? {
        Response::Setup {
            missing_dependencies,
        } => {
            if !missing_dependencies.is_empty() {
                return Err(missing_deps_error(&missing_dependencies));
            }
        }
        Response::Error { message } => return Err(AgentError(message)),
        _ => return Err(AgentError("unexpected response to hello".to_string())),
    }

    Ok(stream)
}

/// Ask the agent to tear down and disconnect. Dropping the connection would also
/// trigger cleanup agent-side (via socket EOF); sending `Shutdown` is the tidy
/// path.
pub fn app_cleanup() {
    if let Some(mut client) = CLIENT.lock().unwrap().take() {
        client.exchange(Request::Shutdown).ok();
    }
}

fn spawn_agent() -> Result<Child, AgentError> {
    let agent = agent_binary_path()?;

    // The agent needs no arguments: it derives the uid from PKEXEC_UID (else
    // getuid()) and the instance id from its parent pid (this GUI).
    let mut command = if geteuid().is_root() {
        // GUI already privileged (e.g. launched with sudo on X11): run the agent
        // directly. Clear any inherited PKEXEC_UID so it resolves the uid from
        // getuid() (0), matching the socket path the GUI connects to.
        let mut command = Command::new(&agent);
        command.env_remove("PKEXEC_UID");
        command
    } else {
        // Normal case: escalate only the agent, via polkit.
        if !deps::is_installed(deps::PKEXEC) {
            return Err(AgentError(
                "could not find 'pkexec' to start the privileged agent, install polkit, or run airgorah as root"
                    .to_string(),
            ));
        }

        let mut command = Command::new(deps::PKEXEC);
        command.arg(&agent);
        command
    };

    command
        .spawn()
        .map_err(|e| AgentError(format!("failed to launch privileged agent: {e}")))
}

fn agent_binary_path() -> Result<PathBuf, AgentError> {
    // The agent must live next to the GUI (both the dev build and the package ship
    // them in the same directory).
    let exe = std::env::current_exe()
        .map_err(|e| AgentError(format!("could not locate the running executable: {e}")))?;

    let candidate = exe
        .parent()
        .ok_or_else(|| AgentError("the running executable has no parent directory".to_string()))?
        .join("airgorah-agent");

    if !candidate.is_file() {
        return Err(AgentError(
            "could not locate the 'airgorah-agent' binary next to the GUI".to_string(),
        ));
    }

    Ok(candidate)
}

fn connect_with_timeout(sock: &str, child: &mut Child) -> Result<UnixStream, AgentError> {
    let deadline = Instant::now() + Duration::from_secs(120);

    loop {
        if let Ok(stream) = UnixStream::connect(sock) {
            return Ok(stream);
        }

        // Fail fast if pkexec/the agent already exited (e.g. auth cancelled).
        if let Ok(Some(status)) = child.try_wait() {
            return Err(AgentError(format!(
                "the privileged agent exited before accepting a connection ({status}), authentication may have been cancelled"
            )));
        }

        if Instant::now() >= deadline {
            return Err(AgentError(
                "timed out waiting for the privileged agent".to_string(),
            ));
        }

        std::thread::sleep(Duration::from_millis(100));
    }
}

// --------------------------------------------------------------------------
// Interface
// --------------------------------------------------------------------------

pub fn enable_monitor_mode(iface: &str) -> Result<String, AgentError> {
    let kill_network_manager = super::get_settings().kill_network_manager;

    match request(Request::EnableMonitor {
        iface: iface.to_string(),
        kill_network_manager,
    })? {
        Response::MonitorEnabled { iface } => Ok(iface),
        Response::Error { message } => Err(AgentError(message)),
        _ => Err(AgentError("unexpected response from agent".to_string())),
    }
}

pub fn set_mac_address(iface: &str) -> Result<(), AgentError> {
    let mac = super::get_settings().mac_mode();
    expect_ok(request(Request::SetMac {
        iface: iface.to_string(),
        mac,
    })?)
}

pub fn disable_monitor_mode(iface: &str) -> Result<(), AgentError> {
    expect_ok(request(Request::DisableMonitor {
        iface: iface.to_string(),
    })?)
}

/// The currently selected monitor interface (GUI-side copy).
pub fn get_iface() -> Option<String> {
    IFACE.lock().unwrap().clone()
}

pub fn set_iface(iface: String) {
    IFACE.lock().unwrap().replace(iface);
}

// --------------------------------------------------------------------------
// Scan
// --------------------------------------------------------------------------

pub fn is_scan_process() -> bool {
    matches!(request(Request::IsScanning), Ok(Response::Bool(true)))
}

pub fn set_scan_process(
    iface: &str,
    ghz_2_4: bool,
    ghz_5: bool,
    channel_filter: Option<String>,
) -> Result<(), AgentError> {
    expect_ok(request(Request::StartScan {
        iface: iface.to_string(),
        ghz_2_4,
        ghz_5,
        channels: channel_filter,
    })?)
}

pub fn stop_scan_process() -> Result<(), AgentError> {
    expect_ok(request(Request::StopScan)?)
}

/// Drop accumulated scan data on both sides (the "restart" action).
pub fn reset_scan_data() {
    request(Request::ResetScanData).ok();
    get_aps().clear();
    get_unlinked_clients().clear();
    get_attack_pool().clear();
    SAVED_HANDSHAKES.lock().unwrap().clear();
}

/// Poll the agent for the current scan snapshot, refresh the local caches, and
/// return the access-point map (matching the old `get_airodump_data`).
pub fn get_airodump_data() -> HashMap<String, AP> {
    let response = match request(Request::GetScanData) {
        Ok(response) => response,
        Err(_) => return get_aps().clone(),
    };

    let (aps_vec, unlinked, attacked) = match response {
        Response::ScanData {
            aps,
            unlinked,
            attacked,
        } => (aps, unlinked, attacked),
        _ => return get_aps().clone(),
    };

    // GUI-side enrichment: merge the saved-handshake overlay onto the snapshot.
    // Vendors are already resolved by the agent, so there is nothing else to fill.
    let overlay = SAVED_HANDSHAKES.lock().unwrap().clone();

    let mut aps_map = HashMap::with_capacity(aps_vec.len());
    for mut ap in aps_vec {
        if let Some(path) = overlay.get(&ap.bssid) {
            ap.saved_handshake = Some(path.clone());
        }
        aps_map.insert(ap.bssid.clone(), ap);
    }

    let mut unlinked_map = HashMap::with_capacity(unlinked.len());
    for client in unlinked {
        unlinked_map.insert(client.mac.clone(), client);
    }

    let mut pool = HashMap::with_capacity(attacked.len());
    for state in attacked {
        pool.insert(state.ap.bssid.clone(), state);
    }

    *get_aps() = aps_map.clone();
    *get_unlinked_clients() = unlinked_map;
    *get_attack_pool() = pool;

    aps_map
}

pub fn get_aps() -> MutexGuard<'static, HashMap<String, AP>> {
    APS.lock().unwrap()
}

pub fn get_unlinked_clients() -> MutexGuard<'static, HashMap<String, Client>> {
    UNLINKED_CLIENTS.lock().unwrap()
}

pub fn get_attack_pool() -> MutexGuard<'static, HashMap<String, AttackState>> {
    ATTACK_POOL.lock().unwrap()
}

/// Record that every currently-captured handshake has been saved to `path`.
/// Replaces the old direct mutation of the AP map so the mark survives the next
/// snapshot refresh (it is stored in the GUI-side overlay).
pub fn mark_handshakes_saved(path: &str) {
    let mut overlay = SAVED_HANDSHAKES.lock().unwrap();
    let mut aps = get_aps();

    for (bssid, ap) in aps.iter_mut() {
        if ap.handshake {
            overlay.insert(bssid.clone(), path.to_string());
            ap.saved_handshake = Some(path.to_string());
        }
    }
}

// --------------------------------------------------------------------------
// Attacks
// --------------------------------------------------------------------------

pub fn launch_deauth_attack(
    _iface: &str,
    ap: AP,
    specific_clients: Option<Vec<String>>,
    software: AttackSoftware,
) -> Result<(), AgentError> {
    expect_ok(request(Request::StartDeauth {
        bssid: ap.bssid,
        clients: specific_clients,
        software,
    })?)
}

pub fn stop_deauth_attack(bssid: &str) {
    request(Request::StopDeauth {
        bssid: bssid.to_string(),
    })
    .ok();
}

// --------------------------------------------------------------------------
// Capture
// --------------------------------------------------------------------------

/// Stream the accumulated capture from the agent and write it to `path` as the
/// user (the agent never writes to a user-chosen path as root), pulling it in
/// bounded chunks so a long capture never has to be held whole in memory.
pub fn save_capture(path: &str) -> Result<(), AgentError> {
    // Create the file lazily, so a failed transfer leaves no empty file behind.
    let mut file: Option<std::fs::File> = None;
    let mut offset: u64 = 0;

    loop {
        match request(Request::GetCaptureChunk { offset })? {
            Response::CaptureChunk { data, last } => {
                let file = match file {
                    Some(ref mut file) => file,
                    None => file.insert(
                        std::fs::File::create(path)
                            .map_err(|e| AgentError(format!("failed to write capture: {e}")))?,
                    ),
                };

                file.write_all(&data)
                    .map_err(|e| AgentError(format!("failed to write capture: {e}")))?;
                offset += data.len() as u64;

                if last {
                    break;
                }
            }
            Response::Error { message } => return Err(AgentError(message)),
            _ => return Err(AgentError("unexpected response from agent".to_string())),
        }
    }

    log::info!("capture saved to '{path}'");

    Ok(())
}

// --------------------------------------------------------------------------
// Misc (unprivileged, same machine)
// --------------------------------------------------------------------------

/// Check if a new version is available.
pub fn check_update(current_version: &str) -> Option<String> {
    let url = "https://api.github.com/repos/martin-olivier/airgorah/releases/latest";

    if let Ok(mut response) = ureq::get(url).call()
        && let Ok(json) = response.body_mut().read_json::<serde_json::Value>()
        && json["tag_name"] != current_version
    {
        let new_version = json["tag_name"].as_str().unwrap_or("unknown").to_owned();

        log::info!("a new version is available: \"{new_version}\"");

        return Some(new_version);
    }

    log::info!("airgorah is up to date");

    None
}

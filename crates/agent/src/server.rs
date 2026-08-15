//! Connection handling and request dispatch.
//!
//! The agent serves exactly one client — the GUI that launched it — over a Unix
//! socket. Every request is authorized by peer credentials before this module is
//! reached (see [`authorized`]), and each request argument that becomes a command
//! line is re-validated here, because the GUI is a lower-trust caller once the
//! privilege boundary exists.

use crate::backend;
use crate::validate::{is_valid_interface_name, is_valid_mac};
use airgorah_common::VERSION;
use airgorah_common::channel::is_valid_channel_filter;
use airgorah_common::deps::{self, Requirer};
use airgorah_common::ipc::*;
use airgorah_common::types::*;

use nix::sys::socket::{getsockopt, sockopt::PeerCredentials};
use std::io;
use std::os::unix::net::UnixStream;

/// Authorize a freshly accepted connection: the peer must be the uid the agent
/// was launched for. Without this, any local user could drive privileged
/// operations (including deauth attacks) through the socket.
pub fn authorized(stream: &UnixStream, expected_uid: u32) -> bool {
    match getsockopt(stream, PeerCredentials) {
        Ok(cred) => {
            if cred.uid() == expected_uid {
                true
            } else {
                log::error!(
                    "rejecting peer uid {} (expected {expected_uid})",
                    cred.uid()
                );
                false
            }
        }
        Err(e) => {
            log::error!("could not read peer credentials: {e}");
            false
        }
    }
}

/// Serve requests until the client disconnects or asks the agent to shut down.
pub fn handle_connection(mut stream: UnixStream) {
    loop {
        let request: Request = match read_msg(&mut stream) {
            Ok(request) => request,
            Err(e) => {
                if e.kind() != io::ErrorKind::UnexpectedEof {
                    log::error!("failed to read request: {e}");
                }
                // EOF: the GUI is gone. Returning triggers cleanup in main().
                break;
            }
        };

        let (response, shutdown) = dispatch(request);

        if let Err(e) = write_msg(&mut stream, &response) {
            log::error!("failed to write response: {e}");
            break;
        }

        if shutdown {
            break;
        }
    }
}

fn err<E: std::fmt::Display>(e: E) -> Response {
    Response::Error {
        message: e.to_string(),
    }
}

/// Handle one request. Returns the response and whether the agent should stop.
fn dispatch(request: Request) -> (Response, bool) {
    match request {
        Request::Hello { version } => {
            if version != VERSION {
                return (
                    err(format!(
                        "protocol version mismatch: agent={VERSION}, gui={version}"
                    )),
                    false,
                );
            }
            (
                Response::Setup {
                    missing_dependencies: deps::missing_required(Requirer::Agent)
                        .into_iter()
                        .map(String::from)
                        .collect(),
                },
                false,
            )
        }

        Request::EnableMonitor {
            iface,
            kill_network_manager,
        } => {
            if !is_valid_interface_name(&iface) {
                return (err("invalid interface name"), false);
            }
            match backend::enable_monitor_mode(&iface, kill_network_manager) {
                Ok(mon_iface) => {
                    backend::set_iface(mon_iface.clone());
                    (Response::MonitorEnabled { iface: mon_iface }, false)
                }
                Err(e) => (err(e), false),
            }
        }

        Request::SetMac { iface, mac } => {
            if !is_valid_interface_name(&iface) {
                return (err("invalid interface name"), false);
            }
            if let MacMode::Specific(ref mac) = mac
                && !is_valid_mac(mac)
            {
                return (err("invalid MAC address"), false);
            }
            match backend::set_mac_address(&iface, &mac) {
                Ok(()) => (Response::Ok, false),
                Err(e) => (err(e), false),
            }
        }

        Request::DisableMonitor { iface } => {
            if !is_valid_interface_name(&iface) {
                return (err("invalid interface name"), false);
            }
            let result = backend::disable_monitor_mode(&iface);
            backend::clear_iface();
            match result {
                Ok(()) => (Response::Ok, false),
                Err(e) => (err(e), false),
            }
        }

        Request::StartScan {
            iface,
            ghz_2_4,
            ghz_5,
            channels,
        } => {
            if !is_valid_interface_name(&iface) {
                return (err("invalid interface name"), false);
            }
            if let Some(ref filter) = channels
                && !is_valid_channel_filter(filter, ghz_2_4, ghz_5)
            {
                return (err("invalid channel filter"), false);
            }
            match backend::set_scan_process(&iface, ghz_2_4, ghz_5, channels) {
                Ok(()) => (Response::Ok, false),
                Err(e) => (err(e), false),
            }
        }

        Request::StopScan => match backend::stop_scan_process() {
            Ok(()) => (Response::Ok, false),
            Err(e) => (err(e), false),
        },

        Request::IsScanning => (Response::Bool(backend::is_scan_process()), false),

        Request::ResetScanData => {
            backend::reset_scan_data();
            (Response::Ok, false)
        }

        Request::GetScanData => {
            let aps: Vec<AP> = backend::get_airodump_data().into_values().collect();
            let unlinked: Vec<Client> = backend::get_unlinked_clients().values().cloned().collect();
            let attacked = backend::get_attack_states();
            (
                Response::ScanData {
                    aps,
                    unlinked,
                    attacked,
                },
                false,
            )
        }

        Request::StartDeauth {
            bssid,
            clients,
            software,
        } => {
            if let Some(ref clients) = clients
                && !clients.iter().all(|c| is_valid_mac(c))
            {
                return (err("invalid client MAC address"), false);
            }
            let iface = match backend::get_iface() {
                Some(iface) => iface,
                None => return (err("no interface selected"), false),
            };
            let ap = match backend::get_aps().get(&bssid).cloned() {
                Some(ap) => ap,
                None => return (err(format!("unknown access point {bssid}")), false),
            };
            if !is_valid_mac(&ap.bssid) {
                return (err("invalid access point BSSID"), false);
            }
            match backend::launch_deauth_attack(&iface, ap, clients, software) {
                Ok(()) => (Response::Ok, false),
                Err(e) => (err(e), false),
            }
        }

        Request::StopDeauth { bssid } => {
            backend::stop_deauth_attack(&bssid);
            (Response::Ok, false)
        }

        Request::StopAllDeauth => {
            backend::stop_all_deauth_attacks();
            (Response::Ok, false)
        }

        Request::GetCaptureChunk { offset } => match backend::get_capture_chunk(offset) {
            Ok((data, last)) => (Response::CaptureChunk { data, last }, false),
            Err(e) => (err(e), false),
        },

        Request::Shutdown => (Response::Ok, true),
    }
}

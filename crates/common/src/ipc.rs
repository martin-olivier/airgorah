//! The IPC contract between the GUI and the privileged agent.
//!
//! Wire format: a 4-byte big-endian length prefix followed by a JSON-encoded
//! [`Request`] or [`Response`].

use crate::types::*;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::io::{self, Read, Write};

/// Directory the agent creates (as root) to hold its listening socket.
pub const RUNTIME_DIR: &str = "/run/airgorah";

/// Hard cap on a single framed message, guarding against a bogus length prefix.
const MAX_MSG_LEN: usize = 64 * 1024 * 1024;

/// Per-instance socket path, keyed by the launching user's uid and the GUI's
/// process id. Each GUI instance gets its own agent and socket, so several
/// instances running at once (e.g. one per wireless card) do not collide.
pub fn socket_path(uid: u32, instance: u32) -> String {
    format!("{RUNTIME_DIR}/{uid}-{instance}.sock")
}

/// A command sent by the GUI to the agent.
#[derive(Debug, Serialize, serde::Deserialize)]
pub enum Request {
    /// First message on a new connection: negotiate the protocol version and
    /// trigger the agent's dependency check.
    Hello {
        version: String,
    },

    // --- interface ---
    // Interface enumeration and 5 GHz capability are unprivileged and handled
    // GUI-side, only monitor-mode control crosses the boundary.
    EnableMonitor {
        iface: String,
        kill_network_manager: bool,
    },
    SetMac {
        iface: String,
        mac: MacMode,
    },
    DisableMonitor {
        iface: String,
    },

    // --- scan ---
    StartScan {
        iface: String,
        ghz_2_4: bool,
        ghz_5: bool,
        channels: Option<String>,
    },
    StopScan,
    IsScanning,
    /// Drop the accumulated access-point / client data (the "restart" action).
    ResetScanData,
    /// Poll for the current merged scan snapshot (sent on the GUI's refresh timer).
    GetScanData,

    // --- attacks ---
    StartDeauth {
        bssid: String,
        clients: Option<Vec<String>>,
        /// Send rounds per second (each round hits every target once).
        rate: u32,
        /// Also send a disassociation frame alongside each deauth.
        disassoc: bool,
    },
    StopDeauth {
        bssid: String,
    },
    StopAllDeauth,

    // --- capture ---
    /// Read one chunk of the saved capture at `offset`; the GUI streams the file
    /// in bounded pieces so a long capture never has to fit in one frame.
    GetCaptureChunk {
        offset: u64,
    },

    /// Ask the agent to clean up and exit.
    Shutdown,
}

/// A reply from the agent to a [`Request`].
#[derive(Debug, Serialize, serde::Deserialize)]
pub enum Response {
    Ok,
    Error {
        message: String,
    },
    /// Reply to [`Request::Hello`]: the required tools the agent found missing
    /// (empty when everything it needs is present).
    Setup {
        missing_dependencies: Vec<String>,
    },
    /// The (possibly renamed) monitor-mode interface name.
    MonitorEnabled {
        iface: String,
    },
    Bool(bool),
    ScanData {
        aps: Vec<AP>,
        unlinked: Vec<Client>,
        attacked: Vec<AttackState>,
    },
    /// One chunk of the capture; `last` marks the final one.
    CaptureChunk {
        data: Vec<u8>,
        last: bool,
    },
}

/// Write a length-prefixed JSON frame.
pub fn write_msg<W: Write, T: Serialize>(w: &mut W, msg: &T) -> io::Result<()> {
    let data =
        serde_json::to_vec(msg).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    if data.len() > MAX_MSG_LEN {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "message exceeds maximum frame length",
        ));
    }

    w.write_all(&(data.len() as u32).to_be_bytes())?;
    w.write_all(&data)?;
    w.flush()
}

/// Read a length-prefixed JSON frame. Returns `UnexpectedEof` on a clean
/// disconnect, which the agent uses as its teardown trigger.
pub fn read_msg<R: Read, T: DeserializeOwned>(r: &mut R) -> io::Result<T> {
    let mut len_buf = [0u8; 4];
    r.read_exact(&mut len_buf)?;

    let len = u32::from_be_bytes(len_buf) as usize;
    if len > MAX_MSG_LEN {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "declared frame length exceeds maximum",
        ));
    }

    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf)?;

    serde_json::from_slice(&buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

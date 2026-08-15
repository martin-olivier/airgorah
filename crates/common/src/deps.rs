//! Catalog of the external command-line tools airgorah relies on.
//!
//! A single source of truth: each tool is declared once, tagged with which
//! process runs it and whether it is required. Both the GUI and the agent derive
//! their required set from this list and check it *in their own environment* —
//! the agent runs as root with `/usr/sbin` on `PATH` and can see tools the
//! unprivileged GUI cannot, so the check must happen on each side.

/// Which airgorah process invokes a given tool.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Requirer {
    /// Only the unprivileged GUI (interface listing, handshake decryption).
    Gui,
    /// Only the privileged agent (monitor mode, scan, deauth, capture).
    Agent,
    /// Both processes — each checks it in its own `PATH`.
    Both,
}

/// An external tool and who needs it.
pub struct Tool {
    pub name: &'static str,
    pub requirer: Requirer,
    pub optional: bool,
}

// Optional tool names, referenced at their on-demand check sites.
pub const MDK4: &str = "mdk4";
pub const CRUNCH: &str = "crunch";
pub const SYSTEMCTL: &str = "systemctl";
pub const PKEXEC: &str = "pkexec";

/// Every external tool airgorah shells out to.
pub const TOOLS: &[Tool] = &[
    // Required.
    Tool {
        name: "sh",
        requirer: Requirer::Both,
        optional: false,
    },
    Tool {
        name: "iw",
        requirer: Requirer::Both,
        optional: false,
    },
    Tool {
        name: "awk",
        requirer: Requirer::Both,
        optional: false,
    },
    Tool {
        name: "aircrack-ng",
        requirer: Requirer::Both,
        optional: false,
    },
    Tool {
        name: "xterm",
        requirer: Requirer::Gui,
        optional: false,
    },
    Tool {
        name: "ip",
        requirer: Requirer::Agent,
        optional: false,
    },
    Tool {
        name: "airmon-ng",
        requirer: Requirer::Agent,
        optional: false,
    },
    Tool {
        name: "airodump-ng",
        requirer: Requirer::Agent,
        optional: false,
    },
    Tool {
        name: "aireplay-ng",
        requirer: Requirer::Agent,
        optional: false,
    },
    Tool {
        name: "mergecap",
        requirer: Requirer::Agent,
        optional: false,
    },
    Tool {
        name: "macchanger",
        requirer: Requirer::Agent,
        optional: false,
    },
    // Optional, checked on demand.
    Tool {
        name: MDK4,
        requirer: Requirer::Agent,
        optional: true,
    },
    Tool {
        name: CRUNCH,
        requirer: Requirer::Gui,
        optional: true,
    },
    Tool {
        name: SYSTEMCTL,
        requirer: Requirer::Both,
        optional: true,
    },
    Tool {
        name: PKEXEC,
        requirer: Requirer::Gui,
        optional: true,
    },
];

/// Whether a tool is available on the current process's `PATH`.
pub fn is_installed(name: &str) -> bool {
    which::which(name).is_ok()
}

/// The required tools for `who` that are not installed, checked in the calling
/// process's own `PATH`. Empty when everything required is present.
pub fn missing_required(who: Requirer) -> Vec<&'static str> {
    TOOLS
        .iter()
        .filter(|tool| !tool.optional && (tool.requirer == who || tool.requirer == Requirer::Both))
        .map(|tool| tool.name)
        .filter(|name| !is_installed(name))
        .collect()
}

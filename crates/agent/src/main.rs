//! `airgorah-agent` — the privileged half of airgorah.
//!
//! Launched by the unprivileged GUI (normally via `pkexec`), it runs as root,
//! listens on a per-instance Unix socket, and performs every operation that needs
//! privilege: interface control, scanning, deauth attacks and capture access. It
//! serves exactly one client, and cleans up all privileged state when that client
//! disconnects — so the wireless card is never left in monitor mode with an
//! orphaned scan running.

mod backend;
mod globals;
mod server;
mod validate;

use airgorah_common::ipc::{RUNTIME_DIR, socket_path};

use nix::unistd::{Uid, chown, geteuid, getppid, getuid};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixListener;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    if !geteuid().is_root() {
        eprintln!("airgorah-agent must run as root");
        std::process::exit(1);
    }

    let uid = resolve_target_uid();
    let instance = resolve_instance();
    let sock_path = socket_path(uid, instance);

    if let Err(e) = prepare_runtime_dir() {
        eprintln!("failed to prepare {RUNTIME_DIR}: {e}");
        std::process::exit(1);
    }

    if let Err(e) = prepare_capture_dir() {
        eprintln!("failed to prepare {}: {e}", globals::CAPTURE_DIR);
        std::process::exit(1);
    }

    // Clear a stale socket left by a previous crashed run.
    std::fs::remove_file(&sock_path).ok();

    let listener = match UnixListener::bind(&sock_path) {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("failed to bind {sock_path}: {e}");
            std::process::exit(1);
        }
    };

    if let Err(e) = secure_socket(&sock_path, uid) {
        eprintln!("failed to secure {sock_path}: {e}");
        std::fs::remove_file(&sock_path).ok();
        std::process::exit(1);
    }

    // Clean up on SIGINT/SIGTERM too, not only on a graceful disconnect.
    let signal_sock = sock_path.clone();
    ctrlc::set_handler(move || {
        backend::app_cleanup();
        std::fs::remove_file(&signal_sock).ok();
        std::process::exit(1);
    })
    .ok();

    // Handshake detection now lives agent-side (it owns the capture files).
    std::thread::spawn(|| {
        loop {
            backend::update_handshakes().ok();
            std::thread::sleep(std::time::Duration::from_millis(1500));
        }
    });

    log::info!("listening on {sock_path} for uid {uid}");

    // Accept exactly one client: the GUI that launched us.
    match listener.accept() {
        Ok((stream, _)) => {
            if server::authorized(&stream, uid) {
                server::handle_connection(stream);
            }
        }
        Err(e) => log::error!("accept failed: {e}"),
    }

    backend::app_cleanup();
    std::fs::remove_file(&sock_path).ok();
    log::info!("exiting");
}

/// Decide which uid the socket belongs to (and which peer is authorized).
///
/// `pkexec` runs us as root but exports `PKEXEC_UID` with the launching user's
/// uid, which is the authoritative source. When the agent is run directly
/// (e.g. by an already-root GUI, with no `PKEXEC_UID`) we fall back to the real
/// uid. Either way the actual authorization is the `SO_PEERCRED` check on the
/// accepted connection.
fn resolve_target_uid() -> u32 {
    std::env::var("PKEXEC_UID")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or_else(|| getuid().as_raw())
}

/// Each GUI instance gets its own socket, keyed by the GUI's process id, so
/// several instances can run at once (e.g. one per wireless card). `pkexec`
/// exec's us in place (and an already-root GUI spawns us directly), so our parent
/// *is* the GUI — its pid is exactly the instance id it built the socket path
/// from.
fn resolve_instance() -> u32 {
    getppid().as_raw() as u32
}

fn prepare_runtime_dir() -> std::io::Result<()> {
    std::fs::create_dir_all(RUNTIME_DIR)?;
    std::fs::set_permissions(RUNTIME_DIR, std::fs::Permissions::from_mode(0o755))?;
    Ok(())
}

/// Create the scan/capture scratch directory, restricted to root (`0700`).
fn prepare_capture_dir() -> std::io::Result<()> {
    std::fs::create_dir_all(globals::CAPTURE_DIR)?;
    std::fs::set_permissions(globals::CAPTURE_DIR, std::fs::Permissions::from_mode(0o700))?;
    Ok(())
}

/// Restrict the socket to its owner and hand ownership to the launching user, so
/// only that user (and root) can connect.
fn secure_socket(path: &str, uid: u32) -> std::io::Result<()> {
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    chown(path, Some(Uid::from_raw(uid)), None)
        .map_err(|e| std::io::Error::from_raw_os_error(e as i32))?;
    Ok(())
}

//! Shared building blocks between the unprivileged `airgorah` GUI and the
//! privileged `airgorah-agent` process.
//!
//! This crate contains only data and pure/utility logic: the wire types, the
//! IPC protocol and its framed-JSON codec, and a couple of utilities that both
//! sides need.

pub mod channel;
pub mod deps;
pub mod handshake;
pub mod ipc;
pub mod types;

pub use types::*;

pub const VERSION: &str = concat!("v", env!("CARGO_PKG_VERSION"));

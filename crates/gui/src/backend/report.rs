use super::{get_aps, get_unlinked_clients};
use airgorah_common::types::{AP, Client};

use serde::Serialize;
use std::fs::File;
use std::io::Write;

#[derive(Debug, Serialize)]
struct Report {
    pub access_points: Vec<AP>,
    pub unlinked_clients: Vec<Client>,
}

#[derive(thiserror::Error, Debug)]
pub enum CapError {
    #[error("Input/Output error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Json error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// Save a JSON report of the current scan snapshot. Serializes the GUI's local
/// caches, so no privileged access is needed.
pub fn save_report(path: &str) -> Result<(), CapError> {
    let access_points = get_aps().values().cloned().collect::<Vec<AP>>();
    let unlinked_clients = get_unlinked_clients()
        .values()
        .cloned()
        .collect::<Vec<Client>>();

    let report = Report {
        access_points,
        unlinked_clients,
    };

    let json_data = serde_json::to_string::<Report>(&report)?;

    let mut file = File::create(path)?;
    file.write_all(json_data.as_bytes())?;

    log::info!("report saved to '{path}'");

    Ok(())
}

//! Vendor (OUI) resolution.
//!
//! The agent produces the client records, so it also resolves each MAC to its
//! manufacturer here — the scan snapshot the GUI receives is then complete, with
//! no field left for the GUI to backfill. The OUI table is generated at build
//! time from `vendors.csv` (see `build.rs`).
//!
//! Resolution runs inline while building a snapshot, cached per MAC so repeated
//! polls do no extra work. The lookup is a handful of hashmap probes, so no
//! background thread is needed.

use crate::globals::VENDORS_CACHE;

use std::collections::HashMap;

include!(concat!(env!("OUT_DIR"), "/vendors.rs"));

/// Resolve a client's MAC address to a vendor name, or `"Unknown"`.
///
/// Walks from the longest usable MAC prefix down, matching against the OUI table
/// (which contains both 24-bit OUIs and longer MA-M/MA-S registrations).
pub fn find_vendor(mac: &str) -> String {
    let mut cache = VENDORS_CACHE.lock().unwrap();

    if let Some(vendor) = cache.get(mac) {
        return vendor.clone();
    }

    let mut prefix = mac.get(..13).unwrap_or(mac).to_string();
    let mut vendor = String::from("Unknown");

    while !prefix.is_empty() {
        if let Some(item) = VENDORS.get(prefix.as_str()) {
            vendor = item.to_string();
            break;
        }
        prefix.pop();
    }

    cache.insert(mac.to_string(), vendor.clone());

    vendor
}

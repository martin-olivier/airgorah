use crate::globals::*;

use std::collections::HashMap;

include!(concat!(env!("OUT_DIR"), "/vendors.rs"));

pub fn find_vendor(mac: &str) -> String {
    let mut vendors = VENDORS_CACHE.lock().unwrap();

    match vendors.get(mac) {
        Some(vendor) => vendor.clone(),
        None => {
            vendors.insert(mac.to_string(), String::new());

            String::new()
        }
    }
}

pub fn update_vendors() {
    let vendors_cache_copy = VENDORS_CACHE.lock().unwrap().clone();

    for (mac, vendor) in vendors_cache_copy {
        if vendor.is_empty() {
            let mut mac_to_find = mac[..13].to_string();
            let mut vendor_name = String::from("Unknown");

            while !mac_to_find.is_empty() {
                if let Some(item) = VENDORS.get(mac_to_find.as_str()) {
                    vendor_name = item.to_string();
                    break;
                }
                mac_to_find.pop();
            }
            VENDORS_CACHE.lock().unwrap().insert(mac, vendor_name);
        }
    }
}

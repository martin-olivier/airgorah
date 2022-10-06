use crate::globals::{IFACE, SCAN_PATH, SCAN_PROC};
use network_interface::NetworkInterfaceConfig;
use std::process::Command;

pub fn get_interfaces() -> Vec<String> {
    let mut ret = vec![];
    let ifaces = network_interface::NetworkInterface::show().unwrap();

    for iface in ifaces {
        if !ret.contains(&iface.name) && iface.name != "lo" {
            ret.push(iface.name);
        }
    }
    ret
}

pub fn enable_monitor_mode(iface: &str) -> Result<String, ()> {
    let output = Command::new("airmon-ng")
        .args(["start", iface])
        .output()
        .expect("failed to execute process: airmon-ng");

    match output.status.success() {
        true => Ok(iface.to_string() + "mon"),
        false => Err(()),
    }
}

pub fn set_scan_process(args: &[&str]) {
    let iface = IFACE.lock().unwrap().to_string();

    match SCAN_PROC.lock().unwrap().as_mut() {
        Some(child) => child.kill().unwrap(),
        None => (),
    };

    std::fs::remove_file(SCAN_PATH.to_string() + "-01.csv").ok();

    let mut proc_args = vec![
        iface.as_str(),
        "-a",
        "--output-format",
        "csv",
        "-w",
        SCAN_PATH,
    ];
    proc_args.append(&mut args.to_vec());

    let child = std::process::Command::new("airodump-ng")
        .args(proc_args)
        .spawn()
        .unwrap();
    SCAN_PROC.lock().unwrap().replace(child);
}

pub fn get_airodump_data() -> Option<String> {
    let full_path = SCAN_PATH.to_string() + "-01.csv";

    let _file = match std::fs::File::open(full_path) {
        Ok(f) => f,
        Err(_) => return None,
    };
    println!("file is OK");
    None
}

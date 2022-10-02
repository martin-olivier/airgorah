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

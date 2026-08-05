//! Validators for values that cross the privilege boundary and end up as
//! arguments to root-run commands.

/// Validate a network interface name.
pub fn is_valid_interface_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 15 {
        return false;
    }

    if name.starts_with('-') || name == "." || name == ".." {
        return false;
    }

    name.bytes()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b'-'))
}

/// Validate a MAC address in the canonical `xx:xx:xx:xx:xx:xx` hexadecimal form.
pub fn is_valid_mac(mac: &str) -> bool {
    let mut groups = 0;

    for part in mac.split(':') {
        if part.len() != 2 || !part.bytes().all(|b| b.is_ascii_hexdigit()) {
            return false;
        }
        groups += 1;
    }

    groups == 6
}

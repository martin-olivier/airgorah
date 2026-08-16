//! Shared raw `AF_PACKET` socket helpers for capturing and injecting 802.11
//! frames on the monitor interface. The capture engine ([`super::sniffer`]) reads
//! radiotap-prefixed frames off it; the deauth engine ([`super::deauth`]) injects
//! them.

use std::ffi::CString;
use std::io;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};

/// Open a raw `AF_PACKET` socket bound to `iface`, seeing/sending every frame.
pub fn open(iface: &str) -> io::Result<OwnedFd> {
    let eth_p_all = (libc::ETH_P_ALL as u16).to_be();

    // SAFETY: standard socket(2) call; the returned fd is immediately wrapped in an
    // OwnedFd so it is closed on drop.
    let fd = unsafe { libc::socket(libc::AF_PACKET, libc::SOCK_RAW, eth_p_all as i32) };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }
    let socket = unsafe { OwnedFd::from_raw_fd(fd) };

    let ifindex = interface_index(iface)?;

    let mut addr: libc::sockaddr_ll = unsafe { std::mem::zeroed() };
    addr.sll_family = libc::AF_PACKET as u16;
    addr.sll_protocol = eth_p_all;
    addr.sll_ifindex = ifindex as i32;

    // SAFETY: bind(2) with a correctly sized sockaddr_ll.
    let ret = unsafe {
        libc::bind(
            socket.as_raw_fd(),
            &addr as *const libc::sockaddr_ll as *const libc::sockaddr,
            std::mem::size_of::<libc::sockaddr_ll>() as libc::socklen_t,
        )
    };
    if ret < 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(socket)
}

/// Set a receive timeout, so a capture loop can hop channels and poll a stop flag.
pub fn set_recv_timeout(socket: &OwnedFd, millis: i64) -> io::Result<()> {
    let timeout = libc::timeval {
        tv_sec: millis / 1000,
        tv_usec: (millis % 1000) * 1000,
    };
    // SAFETY: setsockopt(2) with a correctly sized timeval.
    let ret = unsafe {
        libc::setsockopt(
            socket.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_RCVTIMEO,
            &timeout as *const libc::timeval as *const libc::c_void,
            std::mem::size_of::<libc::timeval>() as libc::socklen_t,
        )
    };
    if ret < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// Receive one frame, returning the number of bytes read (0 on an empty read).
pub fn recv(socket: &OwnedFd, buf: &mut [u8]) -> io::Result<usize> {
    // SAFETY: recv(2) into a buffer of the given length.
    let n = unsafe {
        libc::recv(
            socket.as_raw_fd(),
            buf.as_mut_ptr() as *mut libc::c_void,
            buf.len(),
            0,
        )
    };
    if n < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(n as usize)
}

/// Send one raw frame (radiotap header followed by the 802.11 frame).
pub fn send(socket: &OwnedFd, frame: &[u8]) -> io::Result<()> {
    // SAFETY: send(2) from a buffer of the given length.
    let n = unsafe {
        libc::send(
            socket.as_raw_fd(),
            frame.as_ptr() as *const libc::c_void,
            frame.len(),
            0,
        )
    };
    if n < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// Resolve an interface name to its kernel index.
fn interface_index(iface: &str) -> io::Result<u32> {
    let name = CString::new(iface).map_err(|_| {
        io::Error::new(io::ErrorKind::InvalidInput, "interface name contains a nul")
    })?;
    // SAFETY: if_nametoindex(3) reads a valid C string; returns 0 on error.
    let index = unsafe { libc::if_nametoindex(name.as_ptr()) };
    if index == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(index)
}

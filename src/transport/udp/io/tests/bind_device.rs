//! Tests for `bind_device.rs`: binding the listen socket and the per-peer
//! connected sockets of a UDP instance to one interface (`udp.interface`).
//!
//! The Linux tests read the binding back from the kernel with
//! `getsockopt(SO_BINDTODEVICE)`, so they see what the socket holds, not
//! what the code meant to pass. They bind to `lo`, which every Linux test
//! host has. An unprivileged `SO_BINDTODEVICE` needs kernel 5.7 or later;
//! on an older kernel the process needs `CAP_NET_RAW` and these tests fail
//! to open their sockets rather than pass.
//!
//! macOS and the other Unix hosts compile their tests here but no CI host
//! on Linux runs them.

#[cfg(target_os = "linux")]
use super::bound_interface;
use crate::transport::udp::io::UdpRawSocket;
#[cfg(target_os = "linux")]
use std::os::unix::io::AsRawFd;

const BUF: usize = 65_536;

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn binding_udp_to_a_missing_interface_fails_to_start() {
    let err = UdpRawSocket::open_on_interface(
        "127.0.0.1:0".parse().unwrap(),
        BUF,
        BUF,
        Some("fips-absent-x0"),
    )
    .err()
    .expect("an absent interface cannot be bound");
    assert!(err.to_string().contains("fips-absent-x0"), "{err}");
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
#[test]
fn naming_an_interface_on_a_platform_without_binding_is_refused_as_unsupported() {
    let err =
        UdpRawSocket::open_on_interface("127.0.0.1:0".parse().unwrap(), BUF, BUF, Some("lo0"))
            .err()
            .expect("this platform cannot bind a socket to an interface");
    assert!(
        matches!(err, crate::transport::TransportError::NotSupported(_)),
        "{err}"
    );
}

#[cfg(target_os = "linux")]
#[test]
fn a_listen_socket_bound_to_an_interface_reads_back_that_interface() {
    let sock =
        UdpRawSocket::open_on_interface("127.0.0.1:0".parse().unwrap(), BUF, BUF, Some("lo"))
            .expect("open a listen socket bound to lo");
    assert_eq!(bound_interface(sock.as_raw_fd()).unwrap(), "lo");
}

#[cfg(target_os = "linux")]
#[test]
fn a_connected_peer_socket_is_bound_to_the_instance_interface() {
    let fd = crate::transport::udp::io::open_connected_fd(
        "127.0.0.1:0".parse().unwrap(),
        "127.0.0.1:9".parse().unwrap(),
        BUF,
        BUF,
        Some("lo"),
    )
    .expect("open a connected socket bound to lo");
    assert_eq!(bound_interface(fd.as_raw_fd()).unwrap(), "lo");
}

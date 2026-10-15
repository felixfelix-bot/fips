//! Tests for the interface-name rules in `Config::validate`: the UDP
//! `interface` field, and the Ethernet rules it shares a check with.

use super::*;

/// The validation error `yaml` gives, or `None` if it validates.
fn validation_error(yaml: &str) -> Option<String> {
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    match config.validate() {
        Ok(()) => None,
        Err(ConfigError::Validation(msg)) => Some(msg),
        Err(e) => panic!("expected a validation error, got {e}"),
    }
}

fn udp_on(iface: &str) -> String {
    format!("transports:\n  udp:\n    interface: {iface:?}\n")
}

fn ethernet_on(iface: &str) -> String {
    format!("transports:\n  ethernet:\n    interface: {iface:?}\n")
}

#[test]
fn udp_interface_config_parses() {
    let cfg: crate::config::UdpConfig = serde_yaml::from_str("interface: en0\n").unwrap();
    assert_eq!(cfg.interface.as_deref(), Some("en0"));
    let cfg: crate::config::UdpConfig = serde_yaml::from_str("bind_addr: 0.0.0.0:1\n").unwrap();
    assert!(cfg.interface.is_none());
}

/// An empty name binds a Linux socket to no interface at all, with no
/// error, so the instance would start unbound.
#[test]
fn a_udp_instance_with_an_empty_interface_name_is_rejected() {
    let err = validation_error(&udp_on("")).expect("an empty interface name must be refused");
    assert_eq!(err, "transport `udp` has an empty `interface`");
}

#[test]
fn a_udp_instance_with_an_interface_name_over_fifteen_bytes_is_rejected() {
    let name = "abcdefghijklmnop";
    assert_eq!(name.len(), 16);
    let err = validation_error(&udp_on(name)).expect("a 16-byte interface name must be refused");
    assert!(err.contains("is 16 bytes"), "{err}");
}

#[test]
fn a_fifteen_byte_udp_interface_name_is_accepted() {
    let name = "abcdefghijklmno";
    assert_eq!(name.len(), 15);
    assert_eq!(validation_error(&udp_on(name)), None);
}

#[test]
fn a_named_udp_instance_is_named_in_the_error() {
    let err = validation_error("transports:\n  udp:\n    wlan:\n      interface: \"\"\n")
        .expect("an empty interface name must be refused");
    assert_eq!(err, "transport `wlan` has an empty `interface`");
}

/// Two UDP instances on one interface with different ports are two
/// listeners, not two sockets fighting over one device; the Ethernet
/// duplicate rule does not apply to them.
#[test]
fn two_udp_instances_on_one_interface_are_accepted() {
    let yaml = "transports:\n  udp:\n    a:\n      interface: lo\n      bind_addr: \"127.0.0.1:2121\"\n    b:\n      interface: lo\n      bind_addr: \"127.0.0.1:2122\"\n";
    assert_eq!(validation_error(yaml), None);
}

/// The Ethernet checks now share their syntax rules with UDP; their
/// messages are unchanged.
#[test]
fn ethernet_interface_name_errors_keep_their_messages() {
    assert_eq!(
        validation_error(&ethernet_on("")).unwrap(),
        "transport `ethernet` has an empty `interface`"
    );
    assert_eq!(
        validation_error(&ethernet_on("abcdefghijklmnop")).unwrap(),
        "transport `ethernet` interface `abcdefghijklmnop` is 16 bytes; \
         the kernel limit is 15, so no such interface can exist"
    );
    assert_eq!(
        validation_error(&ethernet_on("eth/0")).unwrap(),
        "transport `ethernet` interface `eth/0` contains a character no \
         interface name may hold"
    );
    assert_eq!(
        validation_error(&ethernet_on("eth 0")).unwrap(),
        "transport `ethernet` interface `eth 0` contains a character no \
         interface name may hold"
    );
    assert_eq!(
        validation_error(
            "transports:\n  ethernet:\n    a:\n      interface: eth0\n    b:\n      interface: eth0\n"
        )
        .map(|e| e.ends_with("both bind interface `eth0`")),
        Some(true)
    );
}

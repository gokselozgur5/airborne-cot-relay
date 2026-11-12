#![allow(clippy::field_reassign_with_default)]
use airborne_cot_relay::{RelayConfig, RelayError};

fn msg(cfg: &RelayConfig) -> String {
    match cfg.validate() {
        Err(RelayError::InvalidConfig(m)) => m,
        other => panic!("expected InvalidConfig, got {other:?}"),
    }
}

#[test]
fn rejects_unparsable_telemetry_addr() {
    let mut cfg = RelayConfig::default();
    cfg.telemetry_listen_addr = "localhost".into();
    assert!(msg(&cfg).contains("telemetry listen addr"));
}

#[test]
fn rejects_out_of_range_port() {
    let mut cfg = RelayConfig::default();
    cfg.cot_broadcast_addr = "1.2.3.4:99999".into();
    assert!(msg(&cfg).contains("cot broadcast addr"));
}

#[test]
fn rejects_port_zero() {
    let mut cfg = RelayConfig::default();
    cfg.telemetry_listen_addr = "0.0.0.0:0".into();
    assert!(msg(&cfg).contains("port"));
}

#[test]
fn accepts_ipv6_socket_addr() {
    let mut cfg = RelayConfig::default();
    cfg.cot_broadcast_addr = "[ff02::1]:6969".into();
    assert!(cfg.validate().is_ok());
}

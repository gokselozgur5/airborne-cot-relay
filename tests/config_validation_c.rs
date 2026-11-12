#![allow(clippy::field_reassign_with_default)]
use airborne_cot_relay::{RelayConfig, RelayError};
use std::collections::HashMap;

fn build(pairs: &[(&str, &str)]) -> Result<RelayConfig, RelayError> {
    let env: HashMap<String, String> = pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    RelayConfig::default().with_env_overrides(|k| env.get(k).cloned())
}

#[test]
fn empty_env_keeps_defaults() {
    assert_eq!(build(&[]).unwrap(), RelayConfig::default());
}

#[test]
fn env_overrides_defaults() {
    let cfg = build(&[
        ("RELAY_TELEMETRY_ADDR", "127.0.0.1:15000"),
        ("RELAY_COT_ADDR", "10.0.0.1:8087"),
        ("RELAY_STALE_TIMEOUT_MS", "250"),
    ])
    .unwrap();
    assert_eq!(cfg.telemetry_listen_addr, "127.0.0.1:15000");
    assert_eq!(cfg.cot_broadcast_addr, "10.0.0.1:8087");
    assert_eq!(cfg.target_stale_timeout_ms, 250);
}

#[test]
fn bad_timeout_names_key() {
    match build(&[("RELAY_STALE_TIMEOUT_MS", "soon")]) {
        Err(RelayError::InvalidConfig(m)) => assert!(m.contains("RELAY_STALE_TIMEOUT_MS")),
        other => panic!("unexpected {other:?}"),
    }
}


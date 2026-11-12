#![allow(clippy::field_reassign_with_default)]
use airborne_cot_relay::{RelayConfig, RelayError};

fn msg(cfg: &RelayConfig) -> String {
    match cfg.validate() {
        Err(RelayError::InvalidConfig(m)) => m,
        other => panic!("expected InvalidConfig, got {other:?}"),
    }
}

#[test]
fn rejects_nan_focal_length() {
    let mut cfg = RelayConfig::default();
    cfg.camera.focal_length_px = f64::NAN;
    assert!(msg(&cfg).contains("focal length"));
}

#[test]
fn rejects_zero_stale_timeout() {
    let mut cfg = RelayConfig::default();
    cfg.target_stale_timeout_ms = 0;
    assert!(msg(&cfg).contains("stale timeout"));
}


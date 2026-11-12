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

#[test]
fn rejects_zero_image_size() {
    let mut cfg = RelayConfig::default();
    cfg.camera.image_width = 0;
    assert!(msg(&cfg).contains("image size"));
}

#[test]
fn rejects_principal_point_outside_image() {
    let mut cfg = RelayConfig::default();
    cfg.camera.principal_u = 2000.0;
    assert!(msg(&cfg).contains("principal point"));
}

#[test]
fn rejects_infinite_ground_elevation() {
    let mut cfg = RelayConfig::default();
    cfg.ground_elevation_m = f64::INFINITY;
    assert!(msg(&cfg).contains("ground elevation"));
}

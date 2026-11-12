//! Smoke tests that exercise every item re-exported from the crate root.

use airborne_cot_relay::{CameraIntrinsics, RelayConfig};

#[test]
fn default_config_validates() {
    let config = RelayConfig::default();
    assert!(config.validate().is_ok());
    assert_eq!(config.target_stale_timeout_ms, 5000);
}

#[test]
fn default_intrinsics_match_documented_values() {
    let intrinsics = CameraIntrinsics::default();
    assert_eq!(intrinsics.focal_length_px, 1200.0);
    assert_eq!(intrinsics.image_width, 1280);
    assert_eq!(intrinsics.image_height, 720);
}

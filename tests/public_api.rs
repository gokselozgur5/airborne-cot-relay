//! Smoke tests that exercise every item re-exported from the crate root.

use airborne_cot_relay::{CameraIntrinsics, RelayConfig};

#[test]
fn default_config_validates() {
    let config = RelayConfig::default();
    assert!(config.validate().is_ok());
    assert_eq!(config.target_stale_timeout_ms, 5000);
}

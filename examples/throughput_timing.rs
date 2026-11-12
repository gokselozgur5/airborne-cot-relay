//! Throughput timing walkthrough.
//!
//! Runs the targeter over a fixed number of frames with one detection per
//! frame and reports wall-clock frames per second. Use `--release` for
//! representative numbers.

use airborne_cot_relay::{PlatformPose, RelayConfig, TargetDetection, Targeter};
use std::time::Instant;

/// Build one fixed nadir detection at the image centre.
fn centre_detection(cfg: &RelayConfig) -> TargetDetection {
    TargetDetection {
        pixel_u: cfg.camera.principal_u,
        pixel_v: cfg.camera.principal_v,
        bbox: (490.0, 490.0, 510.0, 510.0),
        classification: "vehicle".into(),
        confidence: 0.95,
        embedding: None,
    }
}

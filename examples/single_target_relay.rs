//! Single-target relay walkthrough.
//!
//! Feeds one detection through the targeter over several frames, converts the
//! confirmed track into a Cursor-on-Target event and prints the XML.
//! The streamer binds a local loopback socket but sends nothing.

use airborne_cot_relay::{
    CotStreamer, PlatformPose, RelayConfig, TargetDetection, Targeter, TrackSummary,
};

/// Build a level, nadir-looking platform pose at the given time and height.
fn nadir_pose(timestamp_ms: u64, hae_m: f64) -> PlatformPose {
    PlatformPose::new(timestamp_ms, 37.7749, -122.4194, hae_m, 0.0, 0.0, 0.0)
}

fn main() -> Result<(), airborne_cot_relay::RelayError> {
    let cfg = RelayConfig::default();
    let mut targeter = Targeter::with_params(&cfg.camera, cfg.ground_elevation_m, 2, 5, 1.0);
    let streamer = CotStreamer::new("127.0.0.1:0", "127.0.0.1:6969", cfg.target_stale_timeout_ms)?;
    let detection = TargetDetection {
        pixel_u: cfg.camera.principal_u,
        pixel_v: cfg.camera.principal_v,
        bbox: (490.0, 490.0, 510.0, 510.0),
        classification: "vehicle".into(),
        confidence: 0.95,
        embedding: None,
    };

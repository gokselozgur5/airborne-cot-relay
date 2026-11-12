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

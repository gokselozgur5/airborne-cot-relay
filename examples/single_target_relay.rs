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
    for frame in 0..3u64 {
        let pose = nadir_pose(1_000 + frame * 100, 100.0);
        for track in targeter.update(&pose, std::slice::from_ref(&detection))? {
            if !track.is_confirmed {
                continue;
            }
            let summary = TrackSummary {
                id: track.id,
                classification: track.classification.clone(),
                lat_deg: track.lat_deg,
                lon_deg: track.lon_deg,
                hae_m: track.hae_m,
                ce90_m: track.ce90_m,
                le90_m: track.ce90_m,
                course_deg: track.course_deg,
                speed_mps: track.speed_mps,
            };
            println!("{}", streamer.build_cot_xml(&summary, 1_700_000_000_000));
        }
    }
    Ok(())
}

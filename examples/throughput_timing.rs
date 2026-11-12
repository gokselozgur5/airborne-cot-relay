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

fn main() -> Result<(), airborne_cot_relay::RelayError> {
    const FRAMES: u64 = 10_000;
    let cfg = RelayConfig::default();
    let mut targeter = Targeter::with_params(&cfg.camera, cfg.ground_elevation_m, 2, 5, 1.0);
    let detection = centre_detection(&cfg);
    let mut confirmed_updates = 0usize;
    let start = Instant::now();
    for frame in 0..FRAMES {
        let timestamp_ms = 1_000 + frame * 100;
        let pose = PlatformPose::new(timestamp_ms, 37.7749, -122.4194, 100.0, 0.0, 0.0, 0.0);
        let tracks = targeter.update(&pose, std::slice::from_ref(&detection))?;
        confirmed_updates += tracks.iter().filter(|t| t.is_confirmed).count();
    }
    let secs = start.elapsed().as_secs_f64();
    println!(
        "frames={FRAMES} elapsed={secs:.3} s fps={:.0} confirmed_updates={confirmed_updates}",
        FRAMES as f64 / secs
    );
    Ok(())
}

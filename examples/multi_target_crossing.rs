//! Multi-target crossing walkthrough.
//!
//! Two vehicles drive toward each other across the field of view. Each frame
//! is passed to the targeter, and every confirmed track is printed so readers
//! can see the tracker keep the two identities apart.

use airborne_cot_relay::{PlatformPose, RelayConfig, TargetDetection, Targeter};

/// Build a level platform pose at the given time over a fixed reference point.
fn nadir_pose(timestamp_ms: u64) -> PlatformPose {
    PlatformPose::new(timestamp_ms, 37.7749, -122.4194, 100.0, 0.0, 0.0, 0.0)
}

/// Build a detection at the given image column, with a fixed row.
fn detection(pixel_u: f64) -> TargetDetection {
    TargetDetection {
        pixel_u,
        pixel_v: 500.0,
        bbox: (pixel_u - 10.0, 490.0, pixel_u + 10.0, 510.0),
        classification: "vehicle".into(),
        confidence: 0.9,
        embedding: None,
    }
}

fn main() -> Result<(), airborne_cot_relay::RelayError> {
    let cfg = RelayConfig::default();
    let mut targeter = Targeter::with_params(&cfg.camera, cfg.ground_elevation_m, 2, 5, 1.0);
    for step in 0..=8u64 {
        let pose = nadir_pose(1_000 + step * 100);
        // The left object moves right and the right object moves left.
        let left = detection(300.0 + 30.0 * step as f64);
        let right = detection(700.0 - 30.0 * step as f64);
        for track in targeter.update(&pose, &[left, right])? {
            if track.is_confirmed {
                println!(
                    "step {step:>2}: id={} lat={:.6} lon={:.6} ce90={:.2} m",
                    track.id, track.lat_deg, track.lon_deg, track.ce90_m
                );
            }
        }
    }
    Ok(())
}

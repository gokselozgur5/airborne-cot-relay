use airborne_cot_relay::{CameraIntrinsics, PlatformPose, TargetDetection, Targeter};

fn det(u: f64, v: f64) -> TargetDetection {
    let bbox = (u - 10.0, v - 10.0, u + 10.0, v + 10.0);
    let classification = "vehicle".into();
    TargetDetection {
        pixel_u: u,
        pixel_v: v,
        bbox,
        classification,
        confidence: 0.9,
        embedding: None,
    }
}

fn pose(ts: u64) -> PlatformPose {
    PlatformPose::new(ts, 37.0, -122.0, 100.0, 0.0, 0.0, 0.0)
}

#[test]
fn detections_are_tracked_after_a_telemetry_clock_reset() {
    let mut targeter = Targeter::with_params(&CameraIntrinsics::default(), 0.0, 2, 5, 1.0);
    for k in 0..3 {
        targeter
            .update(&pose(100_000 + k * 100), &[det(400.0, 300.0)])
            .unwrap();
    }
    let mut last = Vec::new();
    for k in 0..20 {
        last = targeter
            .update(&pose(500 + k * 100), &[det(900.0, 600.0)])
            .unwrap();
    }
    let seen: Vec<(u64, usize, bool)> = last
        .iter()
        .map(|t| (t.id, t.hits, t.is_confirmed))
        .collect();
    assert!(
        last.iter().any(|t| t.id != 1 && t.is_confirmed),
        "tracks after reset: {seen:?}"
    );
}

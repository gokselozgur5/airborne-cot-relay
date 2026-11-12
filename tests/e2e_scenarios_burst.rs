use airborne_cot_relay::{
    CameraIntrinsics, CotStreamer, PlatformPose, TargetDetection, Targeter, TrackSummary,
    TrackedObject,
};
use std::net::UdpSocket;
use std::time::Duration;

const T0_MS: i64 = 1_700_000_000_000;

fn pose(ts_ms: u64) -> PlatformPose {
    PlatformPose::new(ts_ms, 37.7749, -122.4194, 100.0, 0.0, 0.0, 0.0)
}

fn det(u: f64, v: f64) -> TargetDetection {
    TargetDetection {
        pixel_u: u,
        pixel_v: v,
        bbox: (u - 10.0, v - 10.0, u + 10.0, v + 10.0),
        classification: "vehicle".into(),
        confidence: 0.9,
        embedding: None,
    }
}

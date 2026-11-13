use airborne_cot_relay::pipeline::Pipeline;
use airborne_cot_relay::{PlatformPose, RelayConfig, TargetDetection};

fn pipeline() -> Pipeline {
    Pipeline::from_config(&RelayConfig::default()).unwrap()
}

fn detections(n: usize) -> Vec<TargetDetection> {
    let det = TargetDetection {
        pixel_u: 640.0,
        pixel_v: 360.0,
        bbox: (620.0, 340.0, 660.0, 380.0),
        classification: "vehicle".into(),
        confidence: 0.9,
        embedding: None,
    };
    vec![det; n]
}

fn pose(timestamp_ms: u64) -> PlatformPose {
    PlatformPose {
        timestamp_ms,
        lat_deg: 35.0,
        lon_deg: 139.0,
        hae_m: 120.0,
        roll_rad: 0.0,
        pitch_rad: 0.0,
        yaw_rad: 0.0,
    }
}

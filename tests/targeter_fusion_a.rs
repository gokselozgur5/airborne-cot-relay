use airborne_cot_relay::{CameraIntrinsics, PlatformPose, TargetDetection, Targeter};

fn intrinsics() -> CameraIntrinsics {
    CameraIntrinsics {
        focal_length_px: 1000.0,
        principal_u: 500.0,
        principal_v: 500.0,
        image_width: 1000,
        image_height: 1000,
    }
}

fn detection(u: f64, v: f64, class: &str) -> TargetDetection {
    TargetDetection {
        pixel_u: u,
        pixel_v: v,
        bbox: (u - 10.0, v - 10.0, u + 10.0, v + 10.0),
        classification: class.into(),
        confidence: 0.9,
        embedding: None,
    }
}

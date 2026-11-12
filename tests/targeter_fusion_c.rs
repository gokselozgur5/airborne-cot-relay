use airborne_cot_relay::{CameraIntrinsics, PlatformPose, TargetDetection, Targeter};

fn targeter() -> Targeter {
    let intrinsics = CameraIntrinsics {
        focal_length_px: 1000.0,
        principal_u: 500.0,
        principal_v: 500.0,
        image_width: 1000,
        image_height: 1000,
    };
    Targeter::with_params(&intrinsics, 0.0, 2, 5, 1.0)
}

fn detection(pixel_v: f64) -> TargetDetection {
    TargetDetection {
        pixel_u: 500.0,
        pixel_v,
        bbox: (490.0, 490.0, 510.0, 510.0),
        classification: "vehicle".into(),
        confidence: 0.9,
        embedding: None,
    }
}

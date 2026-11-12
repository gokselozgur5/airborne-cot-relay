use airborne_cot_relay::{CameraIntrinsics, PlatformPose, TargetDetection, Targeter};

const LAT: f64 = 37.7749;
const LON: f64 = -122.4194;
const R_M: f64 = 6378137.0;

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

fn detection(pixel_u: f64) -> TargetDetection {
    TargetDetection {
        pixel_u,
        pixel_v: 500.0,
        bbox: (490.0, 490.0, 510.0, 510.0),
        classification: "vehicle".into(),
        confidence: 0.9,
        embedding: None,
    }
}

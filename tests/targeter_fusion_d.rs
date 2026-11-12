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

fn pose_east_of_origin(east_m: f64, timestamp_ms: u64) -> PlatformPose {
    let lon = LON + (east_m / (R_M * LAT.to_radians().cos())).to_degrees();
    PlatformPose::new(timestamp_ms, LAT, lon, 100.0, 0.0, 0.0, 0.0)
}

fn ground_error_m(lat_deg: f64, lon_deg: f64) -> f64 {
    let north = (lat_deg - LAT).to_radians() * R_M;
    let east = (lon_deg - LON).to_radians() * R_M * LAT.to_radians().cos();
    north.hypot(east)
}

#[test]
fn stationary_target_keeps_one_track_while_platform_moves() {
    let mut t = targeter();
    t.update(&pose_east_of_origin(0.0, 1000), &[detection(500.0)])
        .expect("update 1 failed");
    let tracks = t
        .update(&pose_east_of_origin(10.0, 1100), &[detection(400.0)])
        .expect("update 2 failed");
    assert_eq!(tracks.len(), 1);
    let err_m = ground_error_m(tracks[0].lat_deg, tracks[0].lon_deg);
    assert!(err_m < 1.0, "track is {err_m} m from the target");
}

use airborne_cot_relay::pipeline::summary::track_summary;
use airborne_cot_relay::{GroundIntersection, TrackedObject};

fn track() -> TrackedObject {
    TrackedObject {
        id: 42,
        lat_deg: 35.1,
        lon_deg: 139.2,
        hae_m: 12.5,
        velocity_enu: [3.0, 4.0],
        speed_mps: 5.0,
        course_deg: 36.87,
        ce90_m: 4.25,
        classification: "vehicle".into(),
        hits: 3,
        is_confirmed: true,
    }
}

fn projection(le90_m: f64) -> GroundIntersection {
    GroundIntersection {
        lat_deg: 0.0,
        lon_deg: 0.0,
        hae_m: 0.0,
        ce90_m: 99.0,
        le90_m,
        enu_m: [0.0; 3],
    }
}

#[test]
fn summary_copies_track_fields() {
    let s = track_summary(&track(), &projection(7.5));
    assert_eq!((s.id, s.classification.as_str()), (42, "vehicle"));
    assert_eq!((s.lat_deg, s.lon_deg, s.hae_m), (35.1, 139.2, 12.5));
    assert_eq!((s.speed_mps, s.course_deg), (5.0, 36.87));
    assert_eq!(s.ce90_m, 4.25);
}

#[test]
fn le90_comes_from_the_projection() {
    assert_eq!(track_summary(&track(), &projection(7.5)).le90_m, 7.5);
    assert_eq!(track_summary(&track(), &projection(2.0)).le90_m, 2.0);
}

#[test]
fn pipeline_le90_follows_boresight_projection() {
    use airborne_cot_relay::{pipeline::Pipeline, PlatformPose, RelayConfig};
    let p = Pipeline::from_config(&RelayConfig::default()).unwrap();
    let pose = |hae_m| PlatformPose {
        timestamp_ms: 0,
        lat_deg: 35.0,
        lon_deg: 139.0,
        hae_m,
        roll_rad: 0.0,
        pitch_rad: 0.0,
        yaw_rad: 0.0,
    };
    let low = p.summarize_tracks(&[track()], &pose(100.0)).unwrap();
    let high = p.summarize_tracks(&[track()], &pose(400.0)).unwrap();
    let expected = p.targeter().project_pixel(640.0, 360.0, &pose(100.0));
    assert_eq!(low[0].le90_m, expected.unwrap().le90_m);
    assert_ne!(low[0].le90_m, high[0].le90_m);
}

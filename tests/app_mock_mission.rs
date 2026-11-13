use airborne_cot_relay::app::mock_mission::{generate, MOCK_DURATION_S, MOCK_RATE_HZ};
use airborne_cot_relay::{CameraIntrinsics, TelemetryIngest};

#[test]
fn two_runs_are_identical() {
    let (a, b) = (generate(), generate());
    assert_eq!(a, b);
    let bytes_a: Vec<u8> = a.iter().flat_map(|s| s.frame.clone()).collect();
    let bytes_b: Vec<u8> = b.iter().flat_map(|s| s.frame.clone()).collect();
    assert_eq!(bytes_a, bytes_b);
}

#[test]
fn covers_sixty_seconds_in_order() {
    let m = generate();
    assert_eq!(m.len() as u32, MOCK_DURATION_S * MOCK_RATE_HZ);
    assert!(m.windows(2).all(|w| w[0].timestamp_ms < w[1].timestamp_ms));
    assert!(m.last().unwrap().timestamp_ms - m[0].timestamp_ms >= 59_900);
}

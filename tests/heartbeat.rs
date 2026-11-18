use airborne_cot_relay::heartbeat::{
    HeartbeatConfig, HeartbeatGenerator, PlatformMetrics, PlatformOperationalState, RadioMetrics,
};
use airborne_cot_relay::PlatformPose;

#[test]
fn test_integration_heartbeat_cadence_stream() {
    let cfg = HeartbeatConfig::new("RELAY-99", "STORM-01").with_intervals(500, 1500);
    let pose = PlatformPose::new(5000, 34.0522, -118.2437, 1200.0, 0.05, -0.02, 3.14);
    let plat = PlatformMetrics::new(
        "STORM-01",
        PlatformOperationalState::Nominal,
        92.0,
        24.5,
        48.0,
        15.0,
        500,
        Some(pose),
    );
    let rad = RadioMetrics::new(-58, 26.5, 30, 98, true);
    let mut gen = HeartbeatGenerator::new(cfg, plat, rad);

    let mut generated = Vec::new();
    for t in (0..3000).step_by(250) {
        if let Some(record) = gen.generate_if_due(t) {
            generated.push(record);
        }
    }
    assert_eq!(generated.len(), 6);
    assert_eq!(gen.total_generated(), 6);
    assert_eq!(gen.sequence(), 6);
    assert_eq!(gen.last_generated_ms(), Some(2500));
}

#[test]
fn test_integration_heartbeat_pose_and_radio_update() {
    let cfg = HeartbeatConfig::new("RELAY-100", "VIPER-02");
    let plat = PlatformMetrics::new(
        "VIPER-02",
        PlatformOperationalState::Nominal,
        100.0,
        25.2,
        40.0,
        10.0,
        10,
        None,
    );
    let rad = RadioMetrics::new(-70, 20.0, 20, 80, true);
    let mut gen = HeartbeatGenerator::new(cfg, plat, rad);

    let first = gen.generate(10_000);
    assert!(first.cot_xml.contains(r#"lat="0.0000000" lon="0.0000000""#));

    let updated_pose = PlatformPose::new(10_500, 42.1234567, -71.9876543, 750.5, 0.0, 0.0, 0.0);
    gen.update_pose(Some(updated_pose));
    let mut bad_radio = RadioMetrics::new(-95, 5.0, 10, 30, false);
    bad_radio.record_tx(100, false);
    gen.update_radio(bad_radio);

    let second = gen.generate(11_000);
    assert!(second.cot_xml.contains("42.1234567"));
    assert!(second.cot_xml.contains("-71.9876543"));
    assert!(second.cot_xml.contains("conn=\"0\""));
    assert!(second.cot_xml.contains("q=\"30\""));
}

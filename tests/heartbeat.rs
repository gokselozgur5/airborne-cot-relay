use airborne_cot_relay::heartbeat::{
    HeartbeatConfig, HeartbeatGenerator, PlatformMetrics, PlatformOperationalState, RadioMetrics,
};
use airborne_cot_relay::PlatformPose;

#[test]
fn test_integration_heartbeat_cadence_stream() {
    let cfg = HeartbeatConfig::new("RELAY-99", "STORM-01").with_intervals(500, 1500);
    let pose = PlatformPose::new(5000, 34.0522, -118.2437, 1200.0, 0.05, -0.02, 3.14);
    let plat = PlatformMetrics::new("STORM-01", PlatformOperationalState::Nominal, 92.0, 24.5, 48.0, 15.0, 500, Some(pose));
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

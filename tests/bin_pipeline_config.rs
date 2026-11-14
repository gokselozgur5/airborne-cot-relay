use airborne_cot_relay::pipeline::Pipeline;
use airborne_cot_relay::RelayConfig;

#[test]
fn test_pipeline_builds_from_env_configured_boot() {
    let mut config = RelayConfig::default();
    config.telemetry_listen_addr = "127.0.0.1:14555".to_string();
    config.cot_broadcast_addr = "127.0.0.1:16969".to_string();
    config.target_stale_timeout_ms = 3500;

    let pipeline = Pipeline::from_config(&config).expect("pipeline should build");
    assert_eq!(pipeline.config().telemetry_listen_addr, "127.0.0.1:14555");
    assert_eq!(pipeline.config().cot_broadcast_addr, "127.0.0.1:16969");
    assert_eq!(pipeline.config().target_stale_timeout_ms, 3500);
}

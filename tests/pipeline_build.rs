use airborne_cot_relay::pipeline::Pipeline;
use airborne_cot_relay::{RelayConfig, RelayError};

#[test]
fn default_config_builds_a_pipeline() {
    let pipeline = Pipeline::from_config(&RelayConfig::default()).unwrap();
    assert_eq!(pipeline.config(), &RelayConfig::default());
    assert_eq!(pipeline.metrics().snapshot(0).packets_in, 0);
    assert!(pipeline.latest_pose().is_none());
}

#[test]
fn invalid_config_is_rejected() {
    let mut cfg = RelayConfig::default();
    cfg.telemetry_listen_addr.clear();
    let err = Pipeline::from_config(&cfg).err();
    assert!(matches!(err, Some(RelayError::InvalidConfig(_))));
}

#[test]
fn with_origin_keeps_config_and_clears_pose() {
    let pipeline = Pipeline::from_config(&RelayConfig::default())
        .unwrap()
        .with_origin(35.0, 139.0, 200.0);
    assert_eq!(pipeline.config().target_stale_timeout_ms, 5000);
    assert!(pipeline.latest_pose().is_none());
}

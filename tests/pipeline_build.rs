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

#[test]
fn crate_root_exports_pipeline() {
    let res = airborne_cot_relay::Pipeline::from_config(&RelayConfig::default());
    assert!(res.is_ok());
}

#[test]
fn pipeline_implements_try_from_config() {
    let cfg = RelayConfig::default();
    let p1 = Pipeline::try_from(&cfg);
    assert!(p1.is_ok());
    let p2 = Pipeline::try_from(cfg);
    assert!(p2.is_ok());
}

#[test]
fn pipeline_implements_debug() {
    let pipeline = Pipeline::from_config(&RelayConfig::default()).unwrap();
    let dbg = format!("{:?}", pipeline);
    assert!(dbg.contains("Pipeline"));
}

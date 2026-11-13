use airborne_cot_relay::app::exit::{finish, status_for};
use airborne_cot_relay::metrics::Metrics;
use airborne_cot_relay::RelayError;

#[test]
fn each_variant_has_its_status() {
    assert_eq!(status_for(&RelayError::InvalidConfig("x".into())), 2);
    assert_eq!(status_for(&RelayError::IoError("x".into())), 3);
    assert_eq!(status_for(&RelayError::DecodeError("x".into())), 4);
    assert_eq!(status_for(&RelayError::TargetingError("x".into())), 5);
}

#[test]
fn bind_error_exits_3_and_keeps_the_snapshot() {
    let metrics = Metrics::new();
    metrics.add_packets_in(7);
    let snapshot = metrics.snapshot(0);
    let err: Result<(), _> = Err(RelayError::IoError("failed to bind".into()));
    let out = finish(&err, &snapshot);
    assert_eq!(out.status, 3);
    assert_eq!(out.stdout, snapshot.render());
    assert!(out.stdout.starts_with("packets_in 7\n"));
    assert!(out.stderr.unwrap().contains("IO error: failed to bind"));
}

#[test]
fn success_exits_0_without_stderr() {
    let snapshot = Metrics::new().snapshot(0);
    let out = finish(&Ok::<u64, RelayError>(3), &snapshot);
    assert_eq!(out.status, 0);
    assert_eq!(out.stderr, None);
    assert!(out.stdout.contains("sent 0\n"));
}

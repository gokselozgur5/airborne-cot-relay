use airborne_cot_relay::app::run_mock::run_mock;
use airborne_cot_relay::metrics::Metrics;
use airborne_cot_relay::RelayConfig;
use std::net::UdpSocket;

#[test]
fn mock_run_with_validated_ipv6_target_delivers_or_fails() {
    let rx = UdpSocket::bind("[::1]:0").unwrap();
    let config = RelayConfig {
        cot_broadcast_addr: rx.local_addr().unwrap().to_string(),
        ..RelayConfig::default()
    };
    config.validate().expect("IPv6 target passes validation");
    let metrics = Metrics::new();
    let result = run_mock(&config, &metrics);
    let snap = metrics.snapshot(0);
    assert!(
        result.is_err() || snap.sent > 0,
        "run reported {result:?} with sent={} dropped={}",
        snap.sent,
        snap.dropped
    );
}

use airborne_cot_relay::app::run_mock;
use airborne_cot_relay::config::RelayConfig;
use airborne_cot_relay::metrics::Metrics;
use std::net::UdpSocket;

#[test]
fn test_mission_e2e() {
    let rx = UdpSocket::bind("127.0.0.1:0").unwrap();
    let config = RelayConfig {
        cot_broadcast_addr: rx.local_addr().unwrap().to_string(),
        ..RelayConfig::default()
    };
    let metrics = Metrics::new();
    let sent = run_mock::run_mock(&config, &metrics).unwrap();
    assert!(sent > 0);
}

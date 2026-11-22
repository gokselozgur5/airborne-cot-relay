use airborne_cot_relay::app::run_mock;
use airborne_cot_relay::metrics::Metrics;
use airborne_cot_relay::RelayConfig;
use std::net::UdpSocket;

#[allow(dead_code)]
fn step(lat: f64) -> f64 {
    lat
}

#[test]
fn test_simulated_60_second_flight_mission() {
    let rx = UdpSocket::bind("127.0.0.1:0").unwrap();
    let config = RelayConfig {
        cot_broadcast_addr: rx.local_addr().unwrap().to_string(),
        ..RelayConfig::default()
    };
    assert!(config.validate().is_ok());

    let metrics = Metrics::new();
    let sent = run_mock::run_mock(&config, &metrics).unwrap();
    assert!(sent > 0);

    let mut buf = [0u8; 2048];
    let n = rx.recv(&mut buf).unwrap();
    let cot_str = String::from_utf8_lossy(&buf[..n]);
    assert!(cot_str.contains("uid=\"TRACK-"));
    assert_eq!(metrics.snapshot(0).packets_in, 600);
}

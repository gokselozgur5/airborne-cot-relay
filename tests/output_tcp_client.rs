use airborne_cot_relay::output::{TcpClientConfig, TcpClientSink};
use std::time::Duration;

#[test]
fn test_tcp_client_config_builder() {
    let cfg = TcpClientConfig::new()
        .with_connect_timeout(Duration::from_millis(200))
        .with_backoff(50, 2000);
    assert_eq!(cfg.connect_timeout, Some(Duration::from_millis(200)));
    assert_eq!(cfg.backoff_base_ms, 50);
    assert_eq!(cfg.backoff_max_ms, 2000);
}

use airborne_cot_relay::output::{TcpClientConfig, TcpClientSink};

#[test]
fn test_tcp_client_config_builder() {
    let cfg = TcpClientConfig::new();
    assert_eq!(cfg.backoff_base_ms, 100);
}

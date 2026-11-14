use airborne_cot_relay::output::{TcpClientConfig, TcpClientSink};
use airborne_cot_relay::relay::CotSink;
use std::io::Read;
use std::net::TcpListener;
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

#[test]
fn test_tcp_client_sink_deliver_payload() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let mut sink = TcpClientSink::with_defaults(addr);
    let payload = b"<event uid=\"test-tcp\"/>";
    let sent = sink.send(payload).unwrap();
    assert_eq!(sent, payload.len());
    let (mut stream, _) = listener.accept().unwrap();
    let mut buf = [0u8; 64];
    let n = stream.read(&mut buf).unwrap();
    assert_eq!(&buf[..n], payload);
}

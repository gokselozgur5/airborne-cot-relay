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

#[test]
fn test_tcp_client_sink_connect_failure_backoff() {
    let dummy_addr: std::net::SocketAddr = "127.0.0.1:1".parse().unwrap();
    let cfg = TcpClientConfig::new().with_backoff(25, 500);
    let mut sink = TcpClientSink::new(dummy_addr, cfg);
    assert_eq!(sink.backoff().attempt(), 0);
    let err = sink.send(b"data");
    assert!(err.is_err());
    assert_eq!(sink.backoff().attempt(), 1);
}

#[test]
fn test_tcp_client_sink_fast_fails_during_backoff_window() {
    let dummy_addr: std::net::SocketAddr = "127.0.0.1:1".parse().unwrap();
    let cfg = TcpClientConfig::new().with_backoff(10_000, 20_000);
    let mut sink = TcpClientSink::new(dummy_addr, cfg);
    assert_eq!(sink.backoff().attempt(), 0);
    let err1 = sink.send(b"packet-1");
    assert!(err1.is_err());
    assert_eq!(sink.backoff().attempt(), 1);
    let err2 = sink.send(b"packet-2");
    assert!(err2.is_err());
    assert_eq!(sink.backoff().attempt(), 1);
}

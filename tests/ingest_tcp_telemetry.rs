use airborne_cot_relay::ingest::TcpTelemetrySource;
use airborne_cot_relay::runtime::source::TelemetrySource;
use std::io::Write;
use std::net::TcpListener;
use std::time::Duration;

#[test]
fn test_tcp_telemetry_source_receives_loopback_bytes() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let mut client = TcpTelemetrySource::connect(addr).unwrap();
    let (mut server, _) = listener.accept().unwrap();
    server.write_all(b"serial-frame-data").unwrap();
    for _ in 0..50 {
        if let Some(chunk) = client.poll_chunk().unwrap() {
            assert_eq!(chunk, b"serial-frame-data");
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("timed out waiting for tcp chunk");
}

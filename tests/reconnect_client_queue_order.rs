use airborne_cot_relay::relay::CotSink;
use airborne_cot_relay::{ReconnectClient, ReconnectPolicy};
use std::io::Read;
use std::net::TcpListener;
use std::time::Duration;

#[test]
fn payload_queued_by_failed_send_is_delivered_before_newer_ones() {
    let probe = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = probe.local_addr().unwrap();
    drop(probe);
    let policy = ReconnectPolicy::new().with_initial_backoff(Duration::from_millis(1));
    let mut client = ReconnectClient::new(addr, policy);
    assert!(client.send(b"first;").is_err());
    assert_eq!(client.queued_messages(), 1);
    let listener = TcpListener::bind(addr).unwrap();
    std::thread::sleep(Duration::from_millis(20));
    client.send(b"second;").unwrap();
    let left_in_queue = client.queued_messages();
    client.disconnect();
    let mut got = String::new();
    listener.accept().unwrap().0.read_to_string(&mut got).unwrap();
    assert_eq!((got.as_str(), left_in_queue), ("first;second;", 0));
}

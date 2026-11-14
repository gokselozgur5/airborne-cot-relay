use airborne_cot_relay::output::UdpBroadcastSink;
use airborne_cot_relay::relay::CotSink;
use std::net::{SocketAddr, UdpSocket};

#[test]
fn test_broadcast_sink_deliver_payload() {
    let rx = UdpSocket::bind("127.0.0.1:0").unwrap();
    let rx_addr = rx.local_addr().unwrap();
    let mut sink = UdpBroadcastSink::bind(rx_addr).unwrap();
    let payload = b"<event uid=\"test-bcast\"/>";
    let sent = sink.send(payload).unwrap();
    assert_eq!(sent, payload.len());
    let mut buf = [0u8; 64];
    let (n, _) = rx.recv_from(&mut buf).unwrap();
    assert_eq!(&buf[..n], payload);
}

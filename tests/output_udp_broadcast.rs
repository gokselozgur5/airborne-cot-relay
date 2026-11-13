use airborne_cot_relay::output::{UdpBroadcastConfig, UdpBroadcastSink};
use airborne_cot_relay::relay::CotSink;
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};

#[test]
fn test_broadcast_config_builder() {
    let cfg = UdpBroadcastConfig::new().with_outbound_interface(Ipv4Addr::new(127, 0, 0, 1));
    assert_eq!(cfg.outbound_interface, Some(Ipv4Addr::new(127, 0, 0, 1)));
}

#[test]
fn test_broadcast_sink_options() {
    let target: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let sink = UdpBroadcastSink::with_defaults(target).unwrap();
    assert_eq!(sink.target_addr(), target);
    assert!(sink.is_broadcast());
}

#[test]
fn test_broadcast_sink_deliver_payload() {
    let rx = UdpSocket::bind("127.0.0.1:0").unwrap();
    let rx_addr = rx.local_addr().unwrap();
    let mut sink = UdpBroadcastSink::with_defaults(rx_addr).unwrap();
    let payload = b"<event uid=\"test-bc\"/>";
    let sent = sink.send(payload).unwrap();
    assert_eq!(sent, payload.len());
    let mut buf = [0u8; 64];
    let (n, _) = rx.recv_from(&mut buf).unwrap();
    assert_eq!(&buf[..n], payload);
}

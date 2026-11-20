use airborne_cot_relay::output::{UdpMulticastConfig, UdpMulticastSink};
use airborne_cot_relay::relay::CotSink;
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};

#[test]
fn test_multicast_config_builder() {
    let cfg = UdpMulticastConfig::new()
        .with_ttl(4)
        .with_loopback(true)
        .with_outbound_interface(Ipv4Addr::new(127, 0, 0, 1));
    assert_eq!(cfg.ttl, Some(4));
    assert_eq!(cfg.loopback, Some(true));
    assert_eq!(cfg.outbound_interface, Some(Ipv4Addr::new(127, 0, 0, 1)));
}

#[test]
fn test_multicast_sink_options() {
    let target: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let cfg = UdpMulticastConfig::new().with_ttl(7).with_loopback(true);
    let sink = UdpMulticastSink::bind(target, &cfg).unwrap();
    assert_eq!(sink.target_addr(), target);
    assert_eq!(sink.ttl(), 7);
    assert!(sink.loopback());
}

#[test]
fn test_multicast_sink_deliver_payload() {
    let rx = UdpSocket::bind("127.0.0.1:0").unwrap();
    let rx_addr = rx.local_addr().unwrap();
    let mut sink = UdpMulticastSink::with_defaults(rx_addr).unwrap();
    let payload = b"<event uid=\"test-mc\"/>";
    let sent = sink.send(payload).unwrap();
    assert_eq!(sent, payload.len());
    let mut buf = [0u8; 64];
    let (n, _) = rx.recv_from(&mut buf).unwrap();
    assert_eq!(&buf[..n], payload);
}

#[test]
fn test_multicast_group_delivery_to_receiver() {
    use airborne_cot_relay::multicast_rx::{MulticastReceiver, MulticastRxConfig};
    use std::time::Duration;
    let group: Ipv4Addr = "239.255.42.51".parse().unwrap();
    let temp = match UdpSocket::bind("0.0.0.0:0") { Ok(s) => s, Err(_) => return };
    let port = temp.local_addr().unwrap().port();
    drop(temp);
    let cfg = MulticastRxConfig::new(group, port).with_read_timeout(Duration::from_millis(200));
    let rx = match MulticastReceiver::bind(cfg) { Ok(r) => r, Err(_) => return };
    let scfg = UdpMulticastConfig::new().with_loopback(true);
    let mut sink = match UdpMulticastSink::bind(SocketAddr::from((group, port)), &scfg) {
        Ok(s) => s,
        Err(_) => return,
    };
    let payload = b"<event uid=\"test-mc-rx\"/>";
    if sink.send(payload).is_err() { return; }
    let (data, _) = rx.recv_datagram().unwrap();
    assert_eq!(&data, payload);
}

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

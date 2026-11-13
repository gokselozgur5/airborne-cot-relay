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

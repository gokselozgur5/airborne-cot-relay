use airborne_cot_relay::output::{UdpMulticastConfig, UdpMulticastSink};
use std::net::{Ipv4Addr, SocketAddr};

#[test]
fn outbound_interface_config_sets_multicast_if_v4() {
    let target: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let iface = Ipv4Addr::new(127, 0, 0, 1);
    let cfg = UdpMulticastConfig::new().with_outbound_interface(iface);
    let sink = UdpMulticastSink::bind(target, &cfg).unwrap();
    assert_eq!(sink.multicast_if_v4().unwrap(), iface);
}

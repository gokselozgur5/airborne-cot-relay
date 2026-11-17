use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::time::Duration;

use airborne_cot_relay::multicast_listener::{
    DropPolicy, MulticastListener, MulticastListenerConfig,
};

#[test]
fn test_multicast_listener_integration_drain_and_stats() {
    let group: Ipv4Addr = "239.255.42.1".parse().unwrap();
    let temp = UdpSocket::bind("127.0.0.1:0").unwrap();
    let port = temp.local_addr().unwrap().port();
    drop(temp);

    let cfg = MulticastListenerConfig::new(group, port)
        .with_ring_capacity(8)
        .with_read_timeout(Duration::from_millis(50))
        .with_nonblocking(true)
        .with_drop_policy(DropPolicy::DropOldest);

    let mut listener = MulticastListener::bind(cfg).unwrap();
    let sender = UdpSocket::bind("127.0.0.1:0").unwrap();
    let target = SocketAddr::from(([127, 0, 0, 1], port));


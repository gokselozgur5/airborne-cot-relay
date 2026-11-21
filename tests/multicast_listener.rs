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

    for i in 0..5 {
        let msg = format!("COT_INTEGRATION_{}", i);
        sender.send_to(msg.as_bytes(), target).unwrap();
    }

    std::thread::sleep(Duration::from_millis(15));
    let read_count = listener.poll_socket().unwrap();
    assert_eq!(read_count, 5);

    let packets = listener.drain_packets();
    assert_eq!(packets.len(), 5);
    assert_eq!(packets[0].data, b"COT_INTEGRATION_0");
    assert_eq!(packets[4].data, b"COT_INTEGRATION_4");

    let stats = listener.stats();
    assert_eq!(stats.received_packets, 5);
    assert_eq!(stats.enqueued_packets, 5);
    assert_eq!(stats.total_dropped(), 0);
}

#[test]
fn test_multicast_listener_integration_oversized_drop() {
    let group: Ipv4Addr = "239.255.42.2".parse().unwrap();
    let temp = UdpSocket::bind("127.0.0.1:0").unwrap();
    let port = temp.local_addr().unwrap().port();
    drop(temp);

    let cfg = MulticastListenerConfig::new(group, port)
        .with_max_packet_size(16)
        .with_read_timeout(Duration::from_millis(50))
        .with_nonblocking(true);

    let mut listener = MulticastListener::bind(cfg).unwrap();
    let sender = UdpSocket::bind("127.0.0.1:0").unwrap();
    let target = SocketAddr::from(([127, 0, 0, 1], port));

    sender.send_to(b"SHORT_VALID", target).unwrap();
    sender
        .send_to(b"THIS_IS_DEFINITELY_TOO_LONG_PACKET", target)
        .unwrap();

    std::thread::sleep(Duration::from_millis(15));
    let read_count = listener.poll_socket().unwrap();
    assert_eq!(read_count, 1);

    let stats = listener.stats();
    assert_eq!(stats.received_packets, 2);
    assert_eq!(stats.enqueued_packets, 1);
    assert_eq!(stats.dropped_oversized, 1);
}

#[test]
fn poll_socket_returns_after_draining_with_default_config() {
    let tmp = UdpSocket::bind("127.0.0.1:0").unwrap();
    let port = tmp.local_addr().unwrap().port();
    drop(tmp);
    let cfg = MulticastListenerConfig::new("239.255.77.1".parse().unwrap(), port);
    let mut listener = MulticastListener::bind(cfg).unwrap();
    let sender = UdpSocket::bind("127.0.0.1:0").unwrap();
    sender.send_to(b"<event/>", ("127.0.0.1", port)).unwrap();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(2);
        let mut total = 0;
        while start.elapsed() < timeout {
            match listener.poll_socket() {
                Ok(n) => {
                    total += n;
                    if total > 0 {
                        let _ = tx.send(Ok(total));
                        return;
                    }
                    std::thread::sleep(Duration::from_millis(5));
                }
                Err(e) => {
                    let _ = tx.send(Err(e.to_string()));
                    return;
                }
            }
        }
        let _ = tx.send(Ok(total));
    });
    let got = rx.recv_timeout(Duration::from_secs(3));
    assert_eq!(got, Ok(Ok(1)), "poll_socket did not return within 2 s");
}

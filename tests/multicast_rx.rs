use std::net::UdpSocket;
use std::time::Duration;

use airborne_cot_relay::multicast_rx::{MulticastReceiver, MulticastRxConfig};

#[test]
fn oversized_datagram_is_not_returned_truncated() {
    let tmp = UdpSocket::bind("127.0.0.1:0").unwrap();
    let port = tmp.local_addr().unwrap().port();
    drop(tmp);
    let cfg = MulticastRxConfig::new("239.255.77.2".parse().unwrap(), port)
        .with_read_timeout(Duration::from_secs(1));
    let rx = MulticastReceiver::bind(cfg).unwrap();
    let mut event = b"<event><detail>".to_vec();
    event.resize(2990, b'x');
    event.extend_from_slice(b"</detail></event>");
    let sender = UdpSocket::bind("127.0.0.1:0").unwrap();
    sender.send_to(&event, ("127.0.0.1", port)).unwrap();
    if let Ok((data, _)) = rx.recv_datagram() {
        assert_eq!(data.len(), event.len(), "datagram was silently cut");
    }
}

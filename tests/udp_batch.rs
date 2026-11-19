use airborne_cot_relay::udp_batch::{BatchPolicy, UdpBatchQueue, UdpPacket};
use std::net::UdpSocket;

#[test]
fn failed_send_keeps_or_counts_the_rest_of_the_batch() {
    let rx = UdpSocket::bind("127.0.0.1:0").unwrap();
    let good = rx.local_addr().unwrap();
    let bad = "[::1]:9".parse().unwrap();
    let tx = UdpSocket::bind("127.0.0.1:0").unwrap();
    let mut queue = UdpBatchQueue::new(BatchPolicy::default());
    queue.enqueue(UdpPacket::new(bad, vec![1])).unwrap();
    queue.enqueue(UdpPacket::new(good, vec![2])).unwrap();
    queue.enqueue(UdpPacket::new(good, vec![3])).unwrap();
    assert!(queue.flush_batch(&tx).is_err());
    let stats = queue.stats();
    let accounted = queue.len() + stats.sent_packets + stats.dropped_packets;
    assert_eq!(accounted, 3, "queued={} stats={stats:?}", queue.len());
}

use std::collections::VecDeque;
use std::net::{SocketAddr, UdpSocket};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UdpPacket {
    pub destination: SocketAddr,
    pub payload: Vec<u8>,
}

impl UdpPacket {
    pub fn new(destination: SocketAddr, payload: Vec<u8>) -> Self {
        Self {
            destination,
            payload,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BatchPolicy {
    pub max_batch_packets: usize,
    pub max_batch_bytes: usize,
    pub max_queue_depth: usize,
    pub drop_oldest_on_full: bool,
}

impl Default for BatchPolicy {
    fn default() -> Self {
        Self {
            max_batch_packets: 32,
            max_batch_bytes: 65507,
            max_queue_depth: 1024,
            drop_oldest_on_full: true,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct BatchQueueStats {
    pub enqueued: usize,
    pub sent_packets: usize,
    pub sent_bytes: usize,
    pub dropped_packets: usize,
    pub batches_sent: usize,
}

#[derive(Debug)]
pub struct UdpBatchQueue {
    queue: VecDeque<UdpPacket>,
    policy: BatchPolicy,
    stats: BatchQueueStats,
    queued_bytes: usize,
}

impl UdpBatchQueue {
    pub fn new(policy: BatchPolicy) -> Self {
        Self {
            queue: VecDeque::new(),
            policy,
            stats: BatchQueueStats::default(),
            queued_bytes: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

impl UdpBatchQueue {
    pub fn queued_bytes(&self) -> usize {
        self.queued_bytes
    }

    pub fn stats(&self) -> BatchQueueStats {
        self.stats
    }

    pub fn clear(&mut self) {
        self.queue.clear();
        self.queued_bytes = 0;
    }
}

impl UdpBatchQueue {
    pub fn enqueue(&mut self, packet: UdpPacket) -> Result<(), &'static str> {
        if self.queue.len() >= self.policy.max_queue_depth {
            if self.policy.drop_oldest_on_full {
                if let Some(dropped) = self.queue.pop_front() {
                    self.queued_bytes = self.queued_bytes.saturating_sub(dropped.payload.len());
                    self.stats.dropped_packets += 1;
                }
            } else {
                return Err("queue capacity exceeded");
            }
        }
        self.queued_bytes += packet.payload.len();
        self.stats.enqueued += 1;
        self.queue.push_back(packet);
        Ok(())
    }
}

impl UdpBatchQueue {
    pub fn drain_batch(&mut self) -> Vec<UdpPacket> {
        let mut batch = Vec::new();
        let mut batch_bytes = 0;
        while let Some(front) = self.queue.front() {
            if !batch.is_empty() && batch.len() >= self.policy.max_batch_packets {
                break;
            }
            if !batch.is_empty() && batch_bytes + front.payload.len() > self.policy.max_batch_bytes {
                break;
            }
            let packet = self.queue.pop_front().unwrap();
            batch_bytes += packet.payload.len();
            self.queued_bytes = self.queued_bytes.saturating_sub(packet.payload.len());
            batch.push(packet);
        }
        batch
    }
}

impl UdpBatchQueue {
    pub fn flush_batch(&mut self, socket: &UdpSocket) -> std::io::Result<usize> {
        let batch = self.drain_batch();
        if batch.is_empty() {
            return Ok(0);
        }
        let count = batch.len();
        for packet in batch {
            let n = socket.send_to(&packet.payload, packet.destination)?;
            self.stats.sent_packets += 1;
            self.stats.sent_bytes += n;
        }
        self.stats.batches_sent += 1;
        Ok(count)
    }
}

impl UdpBatchQueue {
    pub fn flush_all(&mut self, socket: &UdpSocket) -> std::io::Result<usize> {
        let mut total = 0;
        while !self.is_empty() {
            let sent = self.flush_batch(socket)?;
            if sent == 0 {
                break;
            }
            total += sent;
        }
        Ok(total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enqueue_and_len() {
        let mut queue = UdpBatchQueue::new(BatchPolicy::default());
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        assert!(queue.is_empty());
        queue.enqueue(UdpPacket::new(addr, vec![1, 2, 3])).unwrap();
        assert_eq!(queue.len(), 1);
        assert_eq!(queue.queued_bytes(), 3);
        assert!(!queue.is_empty());
    }

    #[test]
    fn test_clear_resets_queue() {
        let mut queue = UdpBatchQueue::new(BatchPolicy::default());
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        queue.enqueue(UdpPacket::new(addr, vec![1, 2, 3])).unwrap();
        queue.clear();
        assert_eq!(queue.len(), 0);
        assert_eq!(queue.queued_bytes(), 0);
    }

    #[test]
    fn test_capacity_rejection() {
        let policy = BatchPolicy {
            max_queue_depth: 2,
            drop_oldest_on_full: false,
            ..Default::default()
        };
        let mut queue = UdpBatchQueue::new(policy);
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        queue.enqueue(UdpPacket::new(addr, vec![1])).unwrap();
        queue.enqueue(UdpPacket::new(addr, vec![2])).unwrap();
        assert!(queue.enqueue(UdpPacket::new(addr, vec![3])).is_err());
        assert_eq!(queue.len(), 2);
    }

    #[test]
    fn test_drop_oldest_on_full() {
        let policy = BatchPolicy {
            max_queue_depth: 2,
            drop_oldest_on_full: true,
            ..Default::default()
        };
        let mut queue = UdpBatchQueue::new(policy);
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        queue.enqueue(UdpPacket::new(addr, vec![1, 1])).unwrap();
        queue.enqueue(UdpPacket::new(addr, vec![2, 2])).unwrap();
        queue.enqueue(UdpPacket::new(addr, vec![3, 3])).unwrap();
        assert_eq!(queue.len(), 2);
        assert_eq!(queue.stats().dropped_packets, 1);
        let batch = queue.drain_batch();
        assert_eq!(batch[0].payload, vec![2, 2]);
        assert_eq!(batch[1].payload, vec![3, 3]);
    }

    #[test]
    fn test_drain_batch_packet_limit() {
        let policy = BatchPolicy {
            max_batch_packets: 2,
            ..Default::default()
        };
        let mut queue = UdpBatchQueue::new(policy);
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        for i in 0..5 {
            queue.enqueue(UdpPacket::new(addr, vec![i])).unwrap();
        }
        let b1 = queue.drain_batch();
        assert_eq!(b1.len(), 2);
        assert_eq!(queue.len(), 3);
        let b2 = queue.drain_batch();
        assert_eq!(b2.len(), 2);
        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn test_drain_batch_byte_limit() {
        let policy = BatchPolicy {
            max_batch_bytes: 5,
            ..Default::default()
        };
        let mut queue = UdpBatchQueue::new(policy);
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        queue.enqueue(UdpPacket::new(addr, vec![1, 2, 3])).unwrap();
        queue.enqueue(UdpPacket::new(addr, vec![4, 5, 6])).unwrap();
        let b1 = queue.drain_batch();
        assert_eq!(b1.len(), 1);
        assert_eq!(b1[0].payload, vec![1, 2, 3]);
        let b2 = queue.drain_batch();
        assert_eq!(b2.len(), 1);
        assert_eq!(b2[0].payload, vec![4, 5, 6]);
    }

    #[test]
    fn test_flush_batch_to_socket() {
        let receiver = UdpSocket::bind("127.0.0.1:0").unwrap();
        let target = receiver.local_addr().unwrap();
        let sender = UdpSocket::bind("127.0.0.1:0").unwrap();

        let mut queue = UdpBatchQueue::new(BatchPolicy::default());
        queue.enqueue(UdpPacket::new(target, vec![10, 20])).unwrap();
        queue.enqueue(UdpPacket::new(target, vec![30, 40])).unwrap();

        let count = queue.flush_batch(&sender).unwrap();
        assert_eq!(count, 2);
        assert_eq!(queue.stats().sent_packets, 2);
        assert_eq!(queue.stats().sent_bytes, 4);
        assert_eq!(queue.stats().batches_sent, 1);

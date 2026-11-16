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

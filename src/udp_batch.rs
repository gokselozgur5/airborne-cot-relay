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

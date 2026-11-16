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

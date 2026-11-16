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

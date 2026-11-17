use std::fmt;
use std::io;
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::ring_buf::AtomicRingBuffer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropPolicy {
    DropNewest,
    DropOldest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MulticastPacket {
    pub data: Vec<u8>,
    pub source: SocketAddr,
    pub received_at: Instant,
}

impl MulticastPacket {
    pub fn new(data: Vec<u8>, source: SocketAddr) -> Self {
        Self {
            data,
            source,
            received_at: Instant::now(),
        }
    }

    pub fn with_timestamp(data: Vec<u8>, source: SocketAddr, received_at: Instant) -> Self {
        Self {
            data,
            source,
            received_at,
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

#[derive(Debug)]
pub enum MulticastListenerError {
    InvalidGroup(Ipv4Addr),
    InvalidPort,
    InvalidRingCapacity,
    InvalidPacketSize,
    Io(io::Error),
    RingBufferFull,
}

impl fmt::Display for MulticastListenerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidGroup(ip) => write!(f, "invalid multicast address: {}", ip),
            Self::InvalidPort => write!(f, "multicast port must be non-zero"),
            Self::InvalidRingCapacity => write!(f, "ring buffer capacity must be non-zero"),
            Self::InvalidPacketSize => write!(f, "maximum packet size must be non-zero"),
            Self::Io(err) => write!(f, "multicast listener io error: {}", err),

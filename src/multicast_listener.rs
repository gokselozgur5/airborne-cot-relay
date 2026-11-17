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


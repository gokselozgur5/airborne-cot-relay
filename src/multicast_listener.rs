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
            Self::RingBufferFull => write!(f, "multicast ring buffer full"),
        }
    }
}

impl std::error::Error for MulticastListenerError {}

impl From<io::Error> for MulticastListenerError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MulticastListenerConfig {
    pub multicast_addr: Ipv4Addr,
    pub interface_addr: Ipv4Addr,
    pub port: u16,
    pub ring_capacity: usize,
    pub max_packet_size: usize,
    pub read_timeout: Option<Duration>,
    pub nonblocking: bool,
    pub drop_policy: DropPolicy,
}

impl MulticastListenerConfig {
    pub fn new(multicast_addr: Ipv4Addr, port: u16) -> Self {
        Self {
            multicast_addr,
            interface_addr: Ipv4Addr::UNSPECIFIED,
            port,
            ring_capacity: 128,
            max_packet_size: 4096,
            read_timeout: None,
            nonblocking: false,
            drop_policy: DropPolicy::DropNewest,
        }
    }

    pub fn with_interface(mut self, interface_addr: Ipv4Addr) -> Self {
        self.interface_addr = interface_addr;
        self
    }

    pub fn with_ring_capacity(mut self, capacity: usize) -> Self {
        self.ring_capacity = capacity;
        self
    }

    pub fn with_max_packet_size(mut self, max_size: usize) -> Self {
        self.max_packet_size = max_size;
        self
    }

    pub fn with_read_timeout(mut self, timeout: Duration) -> Self {
        self.read_timeout = Some(timeout);
        self
    }

    pub fn with_nonblocking(mut self, nonblocking: bool) -> Self {
        self.nonblocking = nonblocking;
        self
    }

    pub fn with_drop_policy(mut self, drop_policy: DropPolicy) -> Self {
        self.drop_policy = drop_policy;
        self
    }

    pub fn validate(&self) -> Result<(), MulticastListenerError> {
        if !self.multicast_addr.is_multicast() {
            return Err(MulticastListenerError::InvalidGroup(self.multicast_addr));
        }
        if self.port == 0 {
            return Err(MulticastListenerError::InvalidPort);
        }
        if self.ring_capacity == 0 {
            return Err(MulticastListenerError::InvalidRingCapacity);
        }
        if self.max_packet_size == 0 {
            return Err(MulticastListenerError::InvalidPacketSize);
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct DroppingStats {
    received_packets: AtomicU64,
    received_bytes: AtomicU64,
    enqueued_packets: AtomicU64,
    dropped_overflow: AtomicU64,
    dropped_oversized: AtomicU64,
    dropped_error: AtomicU64,
}

impl DroppingStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_received(&self, bytes: usize) {
        self.received_packets.fetch_add(1, Ordering::Relaxed);
        self.received_bytes.fetch_add(bytes as u64, Ordering::Relaxed);
    }

    pub fn record_enqueued(&self) {
        self.enqueued_packets.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_dropped_overflow(&self) {
        self.dropped_overflow.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_dropped_oversized(&self) {
        self.dropped_oversized.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_dropped_error(&self) {
        self.dropped_error.fetch_add(1, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> DroppingStatsSnapshot {
        DroppingStatsSnapshot {
            received_packets: self.received_packets.load(Ordering::Relaxed),
            received_bytes: self.received_bytes.load(Ordering::Relaxed),
            enqueued_packets: self.enqueued_packets.load(Ordering::Relaxed),
            dropped_overflow: self.dropped_overflow.load(Ordering::Relaxed),
            dropped_oversized: self.dropped_oversized.load(Ordering::Relaxed),
            dropped_error: self.dropped_error.load(Ordering::Relaxed),
        }
    }

    pub fn reset(&self) {
        self.received_packets.store(0, Ordering::Relaxed);
        self.received_bytes.store(0, Ordering::Relaxed);
        self.enqueued_packets.store(0, Ordering::Relaxed);
        self.dropped_overflow.store(0, Ordering::Relaxed);
        self.dropped_oversized.store(0, Ordering::Relaxed);
        self.dropped_error.store(0, Ordering::Relaxed);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DroppingStatsSnapshot {
    pub received_packets: u64,
    pub received_bytes: u64,
    pub enqueued_packets: u64,
    pub dropped_overflow: u64,
    pub dropped_oversized: u64,
    pub dropped_error: u64,
}

impl DroppingStatsSnapshot {
    pub fn total_dropped(&self) -> u64 {
        self.dropped_overflow + self.dropped_oversized + self.dropped_error
    }

    pub fn drop_rate(&self) -> f64 {
        if self.received_packets == 0 {
            0.0
        } else {
            self.total_dropped() as f64 / self.received_packets as f64
        }
    }
}

pub struct MulticastListener {
    socket: UdpSocket,
    config: MulticastListenerConfig,
    ring_buffer: Arc<AtomicRingBuffer<MulticastPacket>>,
    stats: Arc<DroppingStats>,
    recv_buf: Vec<u8>,
}

impl MulticastListener {
    pub fn bind(config: MulticastListenerConfig) -> Result<Self, MulticastListenerError> {
        config.validate()?;
        let bind_addr = SocketAddr::from(([0, 0, 0, 0], config.port));
        let socket = UdpSocket::bind(bind_addr)?;
        socket.join_multicast_v4(&config.multicast_addr, &config.interface_addr)?;
        if let Some(timeout) = config.read_timeout {
            socket.set_read_timeout(Some(timeout))?;
        }
        if config.nonblocking {
            socket.set_nonblocking(true)?;
        }
        let ring_buffer = Arc::new(AtomicRingBuffer::new(config.ring_capacity));
        let stats = Arc::new(DroppingStats::new());
        let recv_buf = vec![0u8; config.max_packet_size + 1];
        Ok(Self {
            socket,
            config,
            ring_buffer,
            stats,
            recv_buf,
        })
    }

    pub fn enqueue(&self, mut packet: MulticastPacket) -> Result<(), MulticastPacket> {
        match self.config.drop_policy {
            DropPolicy::DropNewest => {
                if let Err(dropped) = self.ring_buffer.push(packet) {
                    self.stats.record_dropped_overflow();
                    Err(dropped)
                } else {
                    self.stats.record_enqueued();
                    Ok(())
                }
            }
            DropPolicy::DropOldest => {
                while let Err(ret) = self.ring_buffer.push(packet) {
                    let _ = self.ring_buffer.pop();
                    self.stats.record_dropped_overflow();
                    packet = ret;
                }
                self.stats.record_enqueued();
                Ok(())
            }
        }

//! Reliable Broadcast Transport (RBT) ACK/NACK retransmission tracker.
//!
//! Tracks transmitted broadcast packets, receiver feedback (ACK/NACK/SACK),
//! and schedules retransmissions with configurable backoff and retry caps.

use std::collections::{HashMap, HashSet};
use std::fmt;

/// Configuration for the Reliable Broadcast Transport retransmission tracker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RbtConfig {
    /// Initial retransmission timeout in milliseconds.
    pub retransmit_timeout_ms: u64,
    /// Maximum retransmission attempts before marking packet dropped.
    pub max_retransmissions: u32,
    /// Maximum number of unacknowledged packets to track concurrently.
    pub max_tracked_packets: usize,
    /// Whether NACK immediately queues packet for retransmission.
    pub nack_immediate: bool,
}

impl Default for RbtConfig {
    fn default() -> Self {
        Self {
            retransmit_timeout_ms: 250,
            max_retransmissions: 5,
            max_tracked_packets: 1024,
            nack_immediate: true,
        }
    }
}

impl RbtConfig {
    /// Creates a custom tracker configuration.
    pub const fn new(
        timeout_ms: u64,
        max_retries: u32,
        capacity: usize,
        nack_immediate: bool,
    ) -> Self {
        Self {
            retransmit_timeout_ms: timeout_ms,
            max_retransmissions: max_retries,
            max_tracked_packets: capacity,
            nack_immediate,
        }
    }
}

/// Errors originating from tracker operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrackerError {
    /// Tracker capacity exceeded.
    CapacityExceeded(usize),
    /// Sequence number already tracked.
    DuplicateSequence(u64),
    /// Sequence number not found in tracker.
    PacketNotFound(u64),
}

impl fmt::Display for TrackerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CapacityExceeded(c) => write!(f, "tracker capacity exceeded: {}", c),
            Self::DuplicateSequence(s) => write!(f, "sequence {} already tracked", s),
            Self::PacketNotFound(s) => write!(f, "sequence {} not found", s),
        }
    }
}

impl std::error::Error for TrackerError {}

/// Operational statistics for the retransmission tracker.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TrackerStats {
    /// Total broadcast packets submitted for tracking.
    pub packets_tracked: u64,
    /// Total unique ACKs received.
    pub acks_received: u64,
    /// Total NACKs received.
    pub nacks_received: u64,
    /// Total retransmission events scheduled.
    pub retransmissions_scheduled: u64,
    /// Packets dropped after exceeding maximum retries.
    pub packets_dropped_max_retries: u64,
    /// Packets pruned after full acknowledgment.
    pub packets_completed: u64,
}

/// Record of an active broadcast packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketRecord {
    pub seq: u64,
    pub payload: Vec<u8>,
    pub first_sent_ms: u64,
    pub last_sent_ms: u64,
    pub retransmit_count: u32,
    pub acks: HashSet<String>,
    pub nacks: HashSet<String>,
    pub force_retransmit: bool,
}

impl PacketRecord {
    pub fn new(seq: u64, payload: Vec<u8>, sent_ms: u64) -> Self {
        Self {
            seq,
            payload,
            first_sent_ms: sent_ms,
            last_sent_ms: sent_ms,
            retransmit_count: 0,
            acks: HashSet::new(),
            nacks: HashSet::new(),
            force_retransmit: false,
        }
    }
}

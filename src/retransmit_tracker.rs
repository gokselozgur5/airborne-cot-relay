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

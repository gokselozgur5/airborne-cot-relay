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

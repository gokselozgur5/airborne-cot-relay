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

/// State machine tracking reliable broadcast retransmissions.
#[derive(Debug, Clone)]
pub struct RetransmitTracker {
    config: RbtConfig,
    records: HashMap<u64, PacketRecord>,
    stats: TrackerStats,
}

impl RetransmitTracker {
    pub fn new(config: RbtConfig) -> Self {
        Self {
            config,
            records: HashMap::new(),
            stats: TrackerStats::default(),
        }
    }
    pub fn stats(&self) -> &TrackerStats {
        &self.stats
    }

    pub fn tracked_count(&self) -> usize {
        self.records.len()
    }

    pub fn track_packet(&mut self, seq: u64, payload: Vec<u8>, now_ms: u64) -> Result<(), TrackerError> {
        if self.records.contains_key(&seq) {
            return Err(TrackerError::DuplicateSequence(seq));
        }
        if self.records.len() >= self.config.max_tracked_packets {
            return Err(TrackerError::CapacityExceeded(self.config.max_tracked_packets));
        }
        self.records.insert(seq, PacketRecord::new(seq, payload, now_ms));
        self.stats.packets_tracked += 1;
        Ok(())
    }
    pub fn process_ack(&mut self, seq: u64, receiver_id: &str) -> bool {
        if let Some(record) = self.records.get_mut(&seq) {
            if record.acks.insert(receiver_id.to_string()) {
                self.stats.acks_received += 1;
                record.force_retransmit = false;
                return true;
            }
        }
        false
    }
    pub fn process_nack(&mut self, seq: u64, receiver_id: &str) -> bool {
        if let Some(record) = self.records.get_mut(&seq) {
            record.nacks.insert(receiver_id.to_string());
            self.stats.nacks_received += 1;
            if self.config.nack_immediate {
                record.force_retransmit = true;
            }
            return true;
        }
        false
    }
    pub fn process_selective_ack(&mut self, receiver_id: &str, cumulative_seq: u64, missing_seqs: &[u64]) {
        let missing: HashSet<u64> = missing_seqs.iter().copied().collect();
        let target_seqs: Vec<u64> = self.records.keys().copied().collect();
        for s in target_seqs {
            if s <= cumulative_seq && !missing.contains(&s) {
                self.process_ack(s, receiver_id);
            } else if missing.contains(&s) {
                self.process_nack(s, receiver_id);
            }
        }
    }
    pub fn get_pending_retransmissions(&mut self, now_ms: u64) -> Vec<(u64, Vec<u8>)> {
        let mut expired_keys = Vec::new();
        let mut to_retransmit = Vec::new();

        for (seq, record) in self.records.iter_mut() {
            let elapsed = now_ms.saturating_sub(record.last_sent_ms);
            let should_retransmit = record.force_retransmit
                || (elapsed >= self.config.retransmit_timeout_ms);

            if should_retransmit {
                if record.retransmit_count >= self.config.max_retransmissions {
                    expired_keys.push(*seq);
                } else {
                    record.retransmit_count += 1;
                    record.last_sent_ms = now_ms;
                    record.force_retransmit = false;
                    to_retransmit.push((*seq, record.payload.clone()));
                }
            }
        }
        for seq in expired_keys {
            self.records.remove(&seq);
            self.stats.packets_dropped_max_retries += 1;
        }

        self.stats.retransmissions_scheduled += to_retransmit.len() as u64;
        to_retransmit
    }

    pub fn is_acked_by(&self, seq: u64, receiver_id: &str) -> bool {
        self.records.get(&seq).map_or(false, |r| r.acks.contains(receiver_id))
    }

    pub fn is_fully_acked(&self, seq: u64, expected: &[String]) -> bool {
        self.records.get(&seq).map_or(false, |r| {
            expected.iter().all(|recv| r.acks.contains(recv))
        })
    }
    pub fn prune_acknowledged(&mut self, min_acks: usize) -> usize {
        let before = self.records.len();
        self.records.retain(|_, record| record.acks.len() < min_acks);
        let removed = before - self.records.len();
        self.stats.packets_completed += removed as u64;
        removed
    }

    pub fn prune_expired(&mut self, now_ms: u64, max_age_ms: u64) -> usize {
        let before = self.records.len();
        self.records.retain(|_, record| now_ms.saturating_sub(record.first_sent_ms) < max_age_ms);
        let removed = before - self.records.len();
        self.stats.packets_dropped_max_retries += removed as u64;
        removed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_and_ack() {
        let mut tracker = RetransmitTracker::new(RbtConfig::default());
        let res = tracker.track_packet(1, vec![10, 20], 1000);
        assert!(res.is_ok());
        assert_eq!(tracker.tracked_count(), 1);
        assert_eq!(tracker.stats().packets_tracked, 1);

        assert!(tracker.process_ack(1, "node-1"));
        assert!(!tracker.process_ack(1, "node-1"));
        assert_eq!(tracker.stats().acks_received, 1);
        assert!(tracker.is_acked_by(1, "node-1"));
        assert!(!tracker.is_acked_by(1, "node-2"));
    }
    #[test]
    fn test_nack_immediate_retransmit() {
        let mut tracker = RetransmitTracker::new(RbtConfig::default());
        tracker.track_packet(42, vec![1, 2, 3], 1000).unwrap();

        let pending = tracker.get_pending_retransmissions(1100);
        assert!(pending.is_empty());

        assert!(tracker.process_nack(42, "node-9"));
        assert_eq!(tracker.stats().nacks_received, 1);

        let pending = tracker.get_pending_retransmissions(1150);
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].0, 42);
        assert_eq!(pending[0].1, vec![1, 2, 3]);
    }
    #[test]
    fn test_selective_ack_flow() {
        let mut tracker = RetransmitTracker::new(RbtConfig::default());
        tracker.track_packet(101, vec![1], 1000).unwrap();
        tracker.track_packet(102, vec![2], 1000).unwrap();
        tracker.track_packet(103, vec![3], 1000).unwrap();

        tracker.process_selective_ack("node-a", 103, &[102]);

        assert!(tracker.is_acked_by(101, "node-a"));
        assert!(!tracker.is_acked_by(102, "node-a"));
        assert!(tracker.is_acked_by(103, "node-a"));

        let pending = tracker.get_pending_retransmissions(1050);
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].0, 102);
    }
    #[test]
    fn test_retransmit_timeout_and_max_retries() {
        let cfg = RbtConfig::new(200, 2, 10, false);
        let mut tracker = RetransmitTracker::new(cfg);
        tracker.track_packet(5, vec![99], 1000).unwrap();

        assert!(tracker.get_pending_retransmissions(1100).is_empty());

        let p1 = tracker.get_pending_retransmissions(1250);
        assert_eq!(p1.len(), 1);

        let p2 = tracker.get_pending_retransmissions(1500);
        assert_eq!(p2.len(), 1);

        let p3 = tracker.get_pending_retransmissions(1750);
        assert!(p3.is_empty());
        assert_eq!(tracker.tracked_count(), 0);
        assert_eq!(tracker.stats().packets_dropped_max_retries, 1);
    }

//! Priority queue packet scheduler for emergency Cursor-on-Target telemetry.

use std::collections::VecDeque;

/// Priority tier for Cursor-on-Target telemetry packets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CotPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
    Emergency = 4,
}

impl CotPriority {
    pub const ALL: [CotPriority; 5] = [
        CotPriority::Low,
        CotPriority::Normal,
        CotPriority::High,
        CotPriority::Critical,
        CotPriority::Emergency,
    ];

    pub fn as_tier(&self) -> usize {
        *self as usize
    }

    /// Infers priority tier from a CoT type string.
    pub fn from_cot_type(cot_type: &str) -> Self {
        let t = cot_type.trim();
        if t.starts_with("b-a-o-tbl") || t.starts_with("b-a-o-pan") || t.starts_with("b-d") {
            CotPriority::Emergency
        } else if t.starts_with("a-h") || t.starts_with("b-r") {
            CotPriority::Critical
        } else if t.starts_with("a-f") || t.starts_with("a-n") {
            CotPriority::High
        } else if t.starts_with("a-u") {
            CotPriority::Normal
        } else {
            CotPriority::Normal
        }
    }

    /// Infers priority tier from raw CoT XML content.
    pub fn from_xml_str(xml: &str) -> Self {
        if xml.contains("emergency") || xml.contains("911") || xml.contains("b-a-o-tbl") {
            CotPriority::Emergency
        } else if let Some(idx) = xml.find("type=\"") {
            let rem = &xml[idx + 6..];
            if let Some(end) = rem.find('"') {
                return Self::from_cot_type(&rem[..end]);
            }
            CotPriority::Normal
        } else {
            CotPriority::Normal
        }
    }
}

/// A scheduled packet with metadata and priority tier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrioritizedPacket {
    pub priority: CotPriority,
    pub sequence: u64,
    pub timestamp_ms: u64,
    pub payload: Vec<u8>,
    pub ttl_ms: Option<u64>,
}

impl PrioritizedPacket {
    pub fn new(priority: CotPriority, sequence: u64, timestamp_ms: u64, payload: Vec<u8>) -> Self {
        Self {
            priority,
            sequence,
            timestamp_ms,
            payload,
            ttl_ms: None,
        }
    }

    pub fn with_ttl(mut self, ttl_ms: u64) -> Self {
        self.ttl_ms = Some(ttl_ms);
        self
    }

    pub fn is_expired(&self, current_time_ms: u64) -> bool {
        match self.ttl_ms {
            Some(ttl) => current_time_ms.saturating_sub(self.timestamp_ms) >= ttl,
            None => false,
        }
    }
}

/// Errors occurring during priority packet scheduling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchedulerError {
    QueueFull,
    InvalidPacket,
}

/// Operational statistics for the priority scheduler.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SchedulerStats {
    pub enqueued: [u64; 5],
    pub dequeued: [u64; 5],
    pub dropped: [u64; 5],
    pub expired: u64,
}

/// Priority queue scheduler managing tiered CoT packet traffic.
#[derive(Debug)]
pub struct PriorityPacketScheduler {
    pub(crate) queues: [VecDeque<PrioritizedPacket>; 5],
    pub(crate) max_capacity: usize,
    pub(crate) current_count: usize,
    pub(crate) next_sequence: u64,
    pub(crate) stats: SchedulerStats,
}

impl PriorityPacketScheduler {
    /// Creates a new priority queue scheduler with the specified packet capacity.
    pub fn new(max_capacity: usize) -> Self {
        Self {
            queues: [
                VecDeque::new(),
                VecDeque::new(),
                VecDeque::new(),
                VecDeque::new(),
                VecDeque::new(),
            ],
            max_capacity: max_capacity.max(1),
            current_count: 0,
            next_sequence: 1,
            stats: SchedulerStats::default(),
        }
    }

    pub fn max_capacity(&self) -> usize {
        self.max_capacity
    }

    pub fn len(&self) -> usize {
        self.current_count
    }

    pub fn is_empty(&self) -> bool {
        self.current_count == 0
    }

    pub fn stats(&self) -> &SchedulerStats {
        &self.stats
    }

    pub fn clear(&mut self) {
        for q in &mut self.queues {
            q.clear();
        }
        self.current_count = 0;
    }

    fn evict_lowest_priority(&mut self, incoming: CotPriority) -> bool {
        for tier in 0..incoming.as_tier() {
            if self.queues[tier].pop_front().is_some() {
                self.stats.dropped[tier] = self.stats.dropped[tier].saturating_add(1);
                self.current_count = self.current_count.saturating_sub(1);
                return true;
            }
        }
        false
    }

    /// Enqueues a packet at the specified priority tier, evicting lower-priority packets if full.
    pub fn push(
        &mut self,
        payload: Vec<u8>,
        priority: CotPriority,
        timestamp_ms: u64,
    ) -> Result<u64, SchedulerError> {
        if self.current_count >= self.max_capacity && !self.evict_lowest_priority(priority) {
            return Err(SchedulerError::QueueFull);
        }
        let seq = self.next_sequence;
        self.next_sequence = self.next_sequence.wrapping_add(1);
        let pkt = PrioritizedPacket::new(priority, seq, timestamp_ms, payload);
        self.queues[priority.as_tier()].push_back(pkt);
        self.current_count += 1;
        self.stats.enqueued[priority.as_tier()] =
            self.stats.enqueued[priority.as_tier()].saturating_add(1);
        Ok(seq)
    }

    pub fn push_packet(&mut self, mut pkt: PrioritizedPacket) -> Result<u64, SchedulerError> {
        let prio = pkt.priority;
        if self.current_count >= self.max_capacity && !self.evict_lowest_priority(prio) {
            return Err(SchedulerError::QueueFull);
        }
        let seq = self.next_sequence;
        self.next_sequence = self.next_sequence.wrapping_add(1);
        pkt.sequence = seq;
        self.queues[prio.as_tier()].push_back(pkt);
        self.current_count += 1;
        self.stats.enqueued[prio.as_tier()] = self.stats.enqueued[prio.as_tier()].saturating_add(1);
        Ok(seq)
    }

    pub fn push_cot(&mut self, cot_xml: &str, timestamp_ms: u64) -> Result<u64, SchedulerError> {
        let priority = CotPriority::from_xml_str(cot_xml);
        self.push(cot_xml.as_bytes().to_vec(), priority, timestamp_ms)
    }

    /// Pops the highest-priority packet available, with FIFO order within tiers.
    pub fn pop(&mut self) -> Option<PrioritizedPacket> {
        for tier in (0..=4).rev() {
            if let Some(pkt) = self.queues[tier].pop_front() {
                self.current_count = self.current_count.saturating_sub(1);
                self.stats.dequeued[tier] = self.stats.dequeued[tier].saturating_add(1);
                return Some(pkt);
            }
        }
        None
    }

    /// Peeks at highest-priority packet without dequeuing.
    pub fn peek(&self) -> Option<&PrioritizedPacket> {
        for tier in (0..=4).rev() {
            if let Some(pkt) = self.queues[tier].front() {
                return Some(pkt);
            }
        }
        None
    }

    /// Prunes all packets exceeding their TTL relative to current_time_ms.
    pub fn prune_expired(&mut self, current_time_ms: u64) -> usize {
        let mut pruned = 0;
        for q in &mut self.queues {
            let before = q.len();
            q.retain(|pkt| !pkt.is_expired(current_time_ms));
            pruned += before - q.len();
        }
        self.current_count = self.current_count.saturating_sub(pruned);
        self.stats.expired = self.stats.expired.saturating_add(pruned as u64);
        pruned
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_ordering() {
        assert!(CotPriority::Emergency > CotPriority::Critical);
        assert!(CotPriority::Critical > CotPriority::High);
        assert!(CotPriority::High > CotPriority::Normal);
        assert!(CotPriority::Normal > CotPriority::Low);
    }

    #[test]
    fn test_from_cot_type_and_xml() {
        assert_eq!(CotPriority::from_cot_type("b-a-o-tbl"), CotPriority::Emergency);
        assert_eq!(CotPriority::from_cot_type("a-h-G"), CotPriority::Critical);
        assert_eq!(CotPriority::from_cot_type("a-f-G-E-V"), CotPriority::High);
        assert_eq!(CotPriority::from_cot_type("a-u-G"), CotPriority::Normal);

        let xml_emerg = r#"<event version="2.0" type="b-a-o-tbl"/>"#;
        assert_eq!(CotPriority::from_xml_str(xml_emerg), CotPriority::Emergency);
    }
}

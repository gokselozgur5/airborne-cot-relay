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
}

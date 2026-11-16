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
}

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
}

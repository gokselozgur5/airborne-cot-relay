//! Adaptive bandwidth throttle and CoT message priority filter.
//!
//! Provides dynamic rate limiting, link congestion state tracking,
//! and priority-aware message filtering for Cursor-on-Target telemetry.

use crate::priority_scheduler::CotPriority;
use std::time::{Duration, Instant};

/// Link congestion state evaluated from RTT and packet loss.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CongestionState {
    Optimal = 0,
    Nominal = 1,
    Constrained = 2,
    Congested = 3,
    Critical = 4,
}

impl CongestionState {
    pub fn bandwidth_factor(&self) -> f64 {
        match self {
            CongestionState::Optimal => 1.0,
            CongestionState::Nominal => 0.85,
            CongestionState::Constrained => 0.60,
            CongestionState::Congested => 0.35,
            CongestionState::Critical => 0.15,
        }
    }

    pub fn min_allowed_priority(&self) -> CotPriority {
        match self {
            CongestionState::Optimal | CongestionState::Nominal => CotPriority::Low,
            CongestionState::Constrained => CotPriority::Normal,
            CongestionState::Congested => CotPriority::High,
            CongestionState::Critical => CotPriority::Critical,
        }
    }
}

/// Reason a packet was dropped or rejected by the bandwidth throttle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropReason {
    BelowPriorityFloor,
    BandwidthExhausted,
    EmergencyPreempted,
}

/// Decision made when admitting a CoT message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThrottleDecision {
    Admitted,
    Throttled { wait_ms: u64 },
    Dropped(DropReason),
}

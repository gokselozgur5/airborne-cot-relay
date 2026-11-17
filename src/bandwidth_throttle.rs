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

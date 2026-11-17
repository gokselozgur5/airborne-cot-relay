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

/// Configuration settings for the adaptive bandwidth throttle.
#[derive(Debug, Clone, PartialEq)]
pub struct ThrottleConfig {
    pub max_bandwidth_bps: u64,
    pub min_bandwidth_bps: u64,
    pub burst_allowance_bytes: usize,
    pub emergency_bypass: bool,
    pub window_duration: Duration,
}

impl Default for ThrottleConfig {
    fn default() -> Self {
        Self {
            max_bandwidth_bps: 64_000,
            min_bandwidth_bps: 4_800,
            burst_allowance_bytes: 4_096,
            emergency_bypass: true,
            window_duration: Duration::from_secs(1),
        }
    }
}

impl ThrottleConfig {
    pub fn new(max_bps: u64, min_bps: u64, burst_bytes: usize) -> Self {
        Self {
            max_bandwidth_bps: max_bps,
            min_bandwidth_bps: min_bps,
            burst_allowance_bytes: burst_bytes,
            emergency_bypass: true,
            window_duration: Duration::from_secs(1),
        }
    }

    pub fn with_emergency_bypass(mut self, bypass: bool) -> Self {
        self.emergency_bypass = bypass;
        self
    }
}

/// Operational statistics for the bandwidth throttle.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ThrottleStats {
    pub packets_admitted: u64,
    pub packets_dropped: u64,
    pub bytes_admitted: u64,
    pub bytes_dropped: u64,
    pub admitted_by_priority: [u64; 5],
    pub dropped_by_priority: [u64; 5],
    pub congestion_transitions: u64,
}

/// Adaptive bandwidth throttle and CoT message priority filter.
#[derive(Debug, Clone)]
pub struct BandwidthThrottle {
    config: ThrottleConfig,
    state: CongestionState,
    token_balance: f64,
    last_update: Instant,
    stats: ThrottleStats,
}

impl BandwidthThrottle {
    pub fn new(config: ThrottleConfig) -> Self {
        let initial_tokens = config.burst_allowance_bytes as f64;
        Self {
            config,
            state: CongestionState::Optimal,
            token_balance: initial_tokens,
            last_update: Instant::now(),
            stats: ThrottleStats::default(),
        }
    }

    pub fn config(&self) -> &ThrottleConfig {
        &self.config
    }

    pub fn state(&self) -> CongestionState {
        self.state
    }

    pub fn stats(&self) -> ThrottleStats {
        self.stats
    }
}

impl BandwidthThrottle {
    pub fn effective_bandwidth_bps(&self) -> u64 {
        let span = self
            .config
            .max_bandwidth_bps
            .saturating_sub(self.config.min_bandwidth_bps);
        let dynamic = (span as f64 * self.state.bandwidth_factor()) as u64;
        self.config.min_bandwidth_bps.saturating_add(dynamic)
    }

    pub fn set_congestion_state(&mut self, state: CongestionState) {
        if self.state != state {
            self.state = state;
            self.stats.congestion_transitions += 1;
        }
    }
}

impl BandwidthThrottle {
    pub fn update_link_conditions(&mut self, rtt: Duration, packet_loss_ratio: f64) {
        let old_state = self.state;
        let rtt_ms = rtt.as_millis();
        let new_state = if packet_loss_ratio > 0.25 || rtt_ms > 800 {
            CongestionState::Critical
        } else if packet_loss_ratio > 0.12 || rtt_ms > 450 {
            CongestionState::Congested
        } else if packet_loss_ratio > 0.05 || rtt_ms > 250 {
            CongestionState::Constrained
        } else if packet_loss_ratio > 0.01 || rtt_ms > 120 {
            CongestionState::Nominal
        } else {
            CongestionState::Optimal
        };
        if new_state != old_state {
            self.state = new_state;
            self.stats.congestion_transitions += 1;
        }
    }
}

impl BandwidthThrottle {
    fn replenish_tokens_at(&mut self, now: Instant) {
        let elapsed = now.duration_since(self.last_update).as_secs_f64();
        let bytes_per_sec = (self.effective_bandwidth_bps() as f64) / 8.0;
        let added = elapsed * bytes_per_sec;
        let cap = self.config.burst_allowance_bytes as f64;
        self.token_balance = (self.token_balance + added).min(cap);
        self.last_update = now;
    }

    pub fn reset_stats(&mut self) {
        self.stats = ThrottleStats::default();
    }
}

impl BandwidthThrottle {
    pub fn admit(&mut self, priority: CotPriority, payload_bytes: usize) -> ThrottleDecision {
        self.replenish_tokens_at(Instant::now());

        if self.config.emergency_bypass && priority == CotPriority::Emergency {
            self.stats.packets_admitted += 1;
            self.stats.bytes_admitted += payload_bytes as u64;
            self.stats.admitted_by_priority[priority.as_tier()] += 1;
            self.token_balance -= payload_bytes as f64;
            return ThrottleDecision::Admitted;
        }

        if priority < self.state.min_allowed_priority() {
            self.stats.packets_dropped += 1;
            self.stats.bytes_dropped += payload_bytes as u64;
            self.stats.dropped_by_priority[priority.as_tier()] += 1;
            return ThrottleDecision::Dropped(DropReason::BelowPriorityFloor);
        }

        self.admit_token_budget(priority, payload_bytes)
    }
}

impl BandwidthThrottle {
    fn calculate_wait_ms(&self, deficit: f64) -> u64 {
        let bytes_per_sec = (self.effective_bandwidth_bps() as f64) / 8.0;
        let wait_secs = if bytes_per_sec > 0.0 {
            deficit / bytes_per_sec
        } else {
            1.0
        };
        (wait_secs * 1000.0).ceil() as u64
    }
}

impl BandwidthThrottle {
    fn admit_token_budget(
        &mut self,
        priority: CotPriority,
        payload_bytes: usize,
    ) -> ThrottleDecision {
        let needed = payload_bytes as f64;
        if self.token_balance >= needed {
            self.token_balance -= needed;
            self.stats.packets_admitted += 1;
            self.stats.bytes_admitted += payload_bytes as u64;
            self.stats.admitted_by_priority[priority.as_tier()] += 1;
            ThrottleDecision::Admitted
        } else {
            let wait_ms = self.calculate_wait_ms(needed - self.token_balance);
            if priority <= CotPriority::Normal && self.state >= CongestionState::Constrained {
                self.stats.packets_dropped += 1;
                self.stats.bytes_dropped += payload_bytes as u64;
                self.stats.dropped_by_priority[priority.as_tier()] += 1;
                ThrottleDecision::Dropped(DropReason::BandwidthExhausted)
            } else {
                ThrottleDecision::Throttled { wait_ms }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_congestion_state_factor_and_priority_floor() {
        assert_eq!(CongestionState::Optimal.bandwidth_factor(), 1.0);
        assert_eq!(CongestionState::Critical.bandwidth_factor(), 0.15);
        assert_eq!(
            CongestionState::Optimal.min_allowed_priority(),
            CotPriority::Low
        );
        assert_eq!(
            CongestionState::Constrained.min_allowed_priority(),
            CotPriority::Normal
        );
        assert_eq!(
            CongestionState::Critical.min_allowed_priority(),
            CotPriority::Critical
        );
    }
}

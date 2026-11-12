//! Deduplication and rate limiting for outbound track events.

use std::cell::Cell;
use std::collections::HashMap;

/// Source of monotonic time in milliseconds.
pub trait Clock {
    fn now_ms(&self) -> u64;
}

/// Manually advanced clock for deterministic tests.
#[derive(Debug, Default)]
pub struct ManualClock(Cell<u64>);

impl ManualClock {
    pub fn new(start_ms: u64) -> Self {
        Self(Cell::new(start_ms))
    }

    pub fn advance(&self, delta_ms: u64) {
        self.0.set(self.0.get() + delta_ms);
    }
}

impl Clock for ManualClock {
    fn now_ms(&self) -> u64 {
        self.0.get()
    }
}

/// Limits applied to outbound track events.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FilterConfig {
    /// Minimum time between accepted events for one uid.
    pub min_interval_ms: u64,
    /// Minimum movement in metres between accepted events for one uid.
    pub min_move_m: f64,
    /// Age after which an unseen uid is evicted.
    pub stale_after_ms: u64,
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            min_interval_ms: 1_000,
            min_move_m: 1.0,
            stale_after_ms: 60_000,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Entry {
    last_ms: u64,
    lat: f64,
    lon: f64,
}

/// Per-uid filter that decides whether an outbound event is sent.
#[derive(Debug)]
pub struct TrackFilter<C: Clock> {
    config: FilterConfig,
    clock: C,
    tracks: HashMap<String, Entry>,
}

impl<C: Clock> TrackFilter<C> {
    pub fn new(config: FilterConfig, clock: C) -> Self {
        Self {
            config,
            clock,
            tracks: HashMap::new(),
        }
    }

    pub fn clock(&self) -> &C {
        &self.clock
    }

    pub fn tracked_count(&self) -> usize {
        self.tracks.len()
    }
}

impl<C: Clock> TrackFilter<C> {
    /// Returns true when an event for `uid` at the given position should be sent.
    pub fn accept(&mut self, uid: &str, lat: f64, lon: f64) -> bool {
        let now = self.clock.now_ms();
        if let Some(prev) = self.tracks.get(uid) {
            if now.saturating_sub(prev.last_ms) < self.config.min_interval_ms {
                return false;
            }
        }
        self.tracks.insert(
            uid.to_string(),
            Entry {
                last_ms: now,
                lat,
                lon,
            },
        );
        true
    }
}

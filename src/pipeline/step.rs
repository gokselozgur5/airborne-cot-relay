//! One pipeline tick: drain the outbound queue, send, evict stale filter entries.

/// Counts produced by one `OutboundStep::step` call.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StepReport {
    /// Tracks that reached the sink.
    pub sent: usize,
    /// Tracks the sink rejected or wrote short.
    pub failed: usize,
    /// Tracks held back by the rate and movement filter.
    pub filtered: usize,
    /// Filter entries evicted as stale after the drain.
    pub evicted: usize,
}

use crate::relay::{CotSink, SendHealth};
use crate::track_filter::{Clock, TrackFilter};

/// Output side of the pipeline: rate filter, sink and send health.
#[derive(Debug)]
pub struct OutboundStep<C: Clock, S: CotSink> {
    filter: TrackFilter<C>,
    sink: S,
    health: SendHealth,
}

impl<C: Clock, S: CotSink> OutboundStep<C, S> {
    /// Builds the step from its filter, sink and send health tracker.
    pub fn new(filter: TrackFilter<C>, sink: S, health: SendHealth) -> Self {
        Self {
            filter,
            sink,
            health,
        }
    }
}

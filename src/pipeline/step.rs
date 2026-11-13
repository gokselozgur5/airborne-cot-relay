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

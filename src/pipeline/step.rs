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

impl<C: Clock, S: CotSink> OutboundStep<C, S> {
    /// The rate and movement filter.
    pub fn filter(&self) -> &TrackFilter<C> {
        &self.filter
    }

    /// The sink tracks are sent to.
    pub fn sink(&self) -> &S {
        &self.sink
    }

    /// Consecutive and total send failures.
    pub fn health(&self) -> &SendHealth {
        &self.health
    }
}

use crate::metrics::Metrics;
use crate::relay::{send_tracked, TrackSummary};
impl<C: Clock, S: CotSink> OutboundStep<C, S> {
    /// Filters, renders and sends one track; `None` means the filter held it back.
    fn send_one<F>(
        &mut self,
        track: &TrackSummary,
        now_ms: u64,
        metrics: &Metrics,
        render: &mut F,
    ) -> Option<bool>
    where
        F: FnMut(&TrackSummary, u64) -> String,
    {
        let uid = format!("TRACK-{:04}", track.id);
        if !self.filter.accept(&uid, track.lat_deg, track.lon_deg) {
            return None;
        }
        let xml = render(track, now_ms);
        let ok = send_tracked(&mut self.sink, &mut self.health, xml.as_bytes()).is_ok();
        if ok {
            metrics.record_sent(now_ms);
        } else {
            metrics.add_dropped(1);
        }
        Some(ok)
    }
}

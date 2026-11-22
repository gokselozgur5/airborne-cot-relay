//! Output stage: track filter, CoT encoding and sink send accounting.

use crate::metrics::Metrics;
use crate::relay::{send_tracked, CotSink, SendHealth};
use crate::track_filter::{Clock, TrackFilter};
use crate::TrackSummary;

/// Encodes one track summary as a CoT payload at a time in milliseconds.
pub type CotEncoder = Box<dyn Fn(&TrackSummary, u64) -> String>;

/// Filters outbound tracks and sends the accepted ones through a sink.
pub struct OutputStage<C: Clock> {
    filter: TrackFilter<C>,
    health: SendHealth,
    encode: CotEncoder,
}

impl<C: Clock> OutputStage<C> {
    /// Builds a stage from a filter, a send health tracker and an encoder.
    pub fn new(filter: TrackFilter<C>, health: SendHealth, encode: CotEncoder) -> Self {
        Self {
            filter,
            health,
            encode,
        }
    }
}

impl<C: Clock + std::fmt::Debug> std::fmt::Debug for OutputStage<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OutputStage")
            .field("filter", &self.filter)
            .field("health", &self.health)
            .finish()
    }
}

impl<C: Clock> OutputStage<C> {
    /// Send health fed by every send result.
    pub fn health(&self) -> &SendHealth {
        &self.health
    }

    /// Mutable send health fed by every send result.
    pub fn health_mut(&mut self) -> &mut SendHealth {
        &mut self.health
    }

    /// Track filter deciding which tracks are sent.
    pub fn filter(&self) -> &TrackFilter<C> {
        &self.filter
    }

    /// Mutable track filter deciding which tracks are sent.
    pub fn filter_mut(&mut self) -> &mut TrackFilter<C> {
        &mut self.filter
    }

    /// Evicts stale entries from the filter and returns the count.
    pub fn evict_stale(&mut self) -> usize {
        self.filter.evict_stale()
    }
}

impl<C: Clock> OutputStage<C> {
    /// Filters, encodes and sends one track; true when it was sent.
    pub fn send_one<S: CotSink>(
        &mut self,
        sink: &mut S,
        metrics: &Metrics,
        track: &TrackSummary,
    ) -> bool {
        let now_ms = self.filter.clock().now_ms();
        let uid = format!("TRACK-{:04}", track.id);
        if !self.filter.accept(&uid, track.lat_deg, track.lon_deg) {
            return false;
        }
        let payload = (self.encode)(track, now_ms);
        let sent = send_tracked(sink, &mut self.health, payload.as_bytes()).is_ok();
        if sent {
            metrics.record_sent(now_ms);
        } else {
            metrics.add_dropped(1);
        }
        sent
    }
}

impl<C: Clock> OutputStage<C> {
    /// Sends every track the filter accepts and returns how many were sent.
    ///
    /// Successful sends increment `Metrics::sent`; every send result feeds
    /// the stage `SendHealth`, and rejected tracks never reach the sink.
    pub fn emit<S: CotSink>(
        &mut self,
        sink: &mut S,
        metrics: &Metrics,
        tracks: &[TrackSummary],
    ) -> usize {
        tracks
            .iter()
            .filter(|t| self.send_one(sink, metrics, t))
            .count()
    }
}

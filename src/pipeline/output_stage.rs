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

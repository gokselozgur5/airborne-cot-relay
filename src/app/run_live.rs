//! Runs the live mission loop through ingest, targeter, track filter and CoT output.
use crate::config::{RelayConfig, RelayError};
use crate::metrics::Metrics;
use crate::clock::{Clock, SystemClock};
use crate::targeter::Targeter;
use crate::telemetry::TelemetryIngest;
use crate::track_filter::{FilterConfig, TrackFilter};
use crate::ingest::UdpTelemetrySource;
use crate::runtime::source::TelemetrySource;
use crate::relay::{CotStreamer, TrackSummary};

/// Runs the full live mission loop.
pub fn run_live(config: &RelayConfig, metrics: &Metrics) -> Result<u64, RelayError> {
    println!("Starting live mission relay loop...");
    // TODO: Connect ingest, targeter, track filter and relay_loop here.
    Ok(0)
}

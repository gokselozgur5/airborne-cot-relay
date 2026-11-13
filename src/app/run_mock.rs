//! Runs the mock mission through ingest, targeter, track filter and CoT output.
use crate::config::{RelayConfig, RelayError};
use crate::relay::CotStreamer;
use crate::targeter::Targeter;
use crate::telemetry::TelemetryIngest;
use crate::track_filter::{FilterConfig, ManualClock, TrackFilter};

/// Geodetic origin of the mock telemetry frames.
const MOCK_ORIGIN: (f64, f64, f64) = (37.7749, -122.4194, 0.0);

/// Library stages wired together for one mock run.
struct MockRelay {
    ingest: TelemetryIngest,
    targeter: Targeter,
    filter: TrackFilter<ManualClock>,
    streamer: CotStreamer,
}

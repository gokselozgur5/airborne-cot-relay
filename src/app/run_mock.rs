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

impl MockRelay {
    fn new(config: &RelayConfig) -> Result<Self, RelayError> {
        let (lat, lon, alt) = MOCK_ORIGIN;
        let stale_ms = config.target_stale_timeout_ms;
        Ok(Self {
            ingest: TelemetryIngest::new(lat, lon, alt, stale_ms),
            targeter: Targeter::new(&config.camera, config.ground_elevation_m),
            filter: TrackFilter::new(FilterConfig::default(), ManualClock::new(0)),
            streamer: CotStreamer::new("0.0.0.0:0", &config.cot_broadcast_addr, stale_ms)?,
        })
    }
}

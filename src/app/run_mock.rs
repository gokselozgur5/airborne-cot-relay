//! Runs the mock mission through ingest, targeter, track filter and CoT output.
use crate::config::{RelayConfig, RelayError};
use crate::metrics::Metrics;
use crate::relay::{Clock, CotStreamer, SystemClock, TrackSummary};
use crate::targeter::{Targeter, TrackedObject};
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

/// Converts a fused track into the CoT summary sent on the wire.
fn to_summary(t: &TrackedObject) -> TrackSummary {
    TrackSummary {
        id: t.id,
        classification: t.classification.clone(),
        lat_deg: t.lat_deg,
        lon_deg: t.lon_deg,
        hae_m: t.hae_m,
        ce90_m: t.ce90_m,
        le90_m: 10.0,
        course_deg: t.course_deg,
        speed_mps: t.speed_mps,
    }
}

impl MockRelay {
    /// Sends one track stamped with wall time; returns 1 when it was sent.
    fn emit(&self, track: &TrackedObject, metrics: &Metrics) -> u64 {
        let now_ms = SystemClock.now_ms();
        match self.streamer.send_track(&to_summary(track), now_ms) {
            Ok(_) => {
                metrics.record_sent(now_ms.max(0) as u64);
                1
            }
            Err(_) => {
                metrics.add_dropped(1);
                0
            }
        }
    }
}

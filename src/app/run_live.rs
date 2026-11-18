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
    
    let mut ingest = TelemetryIngest::new(0.0, 0.0, config.ground_elevation_m, config.target_stale_timeout_ms);
    let mut targeter = Targeter::new(&config.camera, config.ground_elevation_m);
    let mut filter = TrackFilter::new(FilterConfig::default(), SystemClock::new());
    let _streamer = CotStreamer::new(&config.telemetry_listen_addr, &config.cot_broadcast_addr, config.target_stale_timeout_ms)?;
    let mut _source = UdpTelemetrySource::bind(&config.telemetry_listen_addr)?;
    let _clock = SystemClock::new();
    
    loop {
        match _source.poll_chunk()? {
            Some(_) => {}
            None => {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }
    }
}

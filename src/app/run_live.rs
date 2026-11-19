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
            Some(bytes) => {
                metrics.add_packets_in(1);
                let poses = ingest.process_bytes(&bytes);
                for pose in poses {
                    if let Ok(tracks) = targeter.update(&pose, &[]) {
                        for t in tracks.iter().filter(|t| t.is_confirmed) {
                            let uid = format!("TRACK-{:04}", t.id);
                            if filter.accept(&uid, t.lat_deg, t.lon_deg) {
                                let summary = TrackSummary {
                                    id: t.id,
                                    classification: t.classification.clone(),
                                    lat_deg: t.lat_deg,
                                    lon_deg: t.lon_deg,
                                    hae_m: t.hae_m,
                                    ce90_m: t.ce90_m,
                                    le90_m: 10.0,
                                    course_deg: t.course_deg,
                                    speed_mps: t.speed_mps,
                                };
                                let now_ms = clock.now_ms() as i64;
                                if streamer.send_track(&summary, now_ms).is_ok() {
                                    metrics.record_sent(now_ms as u64);
                                } else {
                                    metrics.add_dropped(1);
                                }
                            }
                        }
                    }
                }
            }
            None => {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }
    }
}

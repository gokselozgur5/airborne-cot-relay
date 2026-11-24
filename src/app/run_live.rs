//! Live mission execution wiring real UDP sockets, pipeline, and optional HTTP service.

use crate::clock::Clock;
use crate::config::{RelayConfig, RelayError};
use crate::http::server::{HttpServer, Router};
use crate::http::HttpResponse;
use crate::ingest::udp_detections::UdpDetectionSource;
use crate::ingest::udp_telemetry::UdpTelemetrySource;
use crate::metrics::Metrics;
use crate::output::UdpBroadcastSink;
use crate::pipeline::step::OutboundStep;
use crate::pipeline::Pipeline;
use crate::relay::{BoundedQueue, DropPolicy, SendHealth, SystemClock, TrackSummary};
use crate::runtime::limit::RunLimit;
use crate::runtime::relay_loop::RelayLoop;
use crate::runtime::ShutdownSignal;
use crate::track_filter::{FilterConfig, TrackFilter};
use std::net::SocketAddr;
use std::sync::Arc;

/// Runs airborne-cot-relay in live mode with actual UDP ingest, targeting, and CoT broadcast.
pub fn run_live(
    config: &RelayConfig,
    metrics: &Arc<Metrics>,
    shutdown: ShutdownSignal,
    limit: RunLimit,
) -> Result<u64, RelayError> {
    config.validate()?;

    let telem_source = UdpTelemetrySource::bind(&config.telemetry_listen_addr)?;

    let det_source = if let Some(ref det_addr) = config.detections_listen_addr {
        Some(UdpDetectionSource::bind(det_addr)?)
    } else {
        None
    };

    let target_addr: SocketAddr = config
        .cot_broadcast_addr
        .parse()
        .map_err(|e| RelayError::InvalidConfig(format!("invalid cot_broadcast_addr: {e}")))?;
    let cot_sink = UdpBroadcastSink::bind(target_addr)?;

    let mut http_server = if let Some(ref http_addr) = config.http_listen_addr {
        let mut router = Router::new();
        router.add_route("/health", move |_| HttpResponse::ok("OK\n", "text/plain"));
        let m_metrics = Arc::clone(metrics);
        router.add_route("/metrics", move |_| {
            let now_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            let body = m_metrics.snapshot(now_ms).render();
            HttpResponse::ok(body, "text/plain")
        });
        let server = HttpServer::bind(http_addr, router)
            .map_err(|e| RelayError::IoError(format!("cannot bind HTTP {http_addr}: {e}")))?;
        Some(server)
    } else {
        None
    };

    let filter_config = FilterConfig {
        min_interval_ms: 0,
        stale_after_ms: config.target_stale_timeout_ms,
        ..Default::default()
    };
    let outbound = OutboundStep::new(
        TrackFilter::new(filter_config, SystemClock),
        cot_sink,
        SendHealth::new(3),
    );

    let pipeline = Pipeline::from_config(config)?;
    let queue = BoundedQueue::new(64, DropPolicy::DropOldest);

    let stale_timeout_ms = config.target_stale_timeout_ms;
    let render = move |track: &TrackSummary, now_ms: u64| {
        let cot_type = crate::relay::classification_to_cot_type(&track.classification);
        let stale_ms = now_ms.saturating_add(stale_timeout_ms);
        let time_str = sightline_cot::cot::format_iso8601_utc(now_ms as i64);
        let stale_str = sightline_cot::cot::format_iso8601_utc(stale_ms as i64);
        let uid = format!("TRACK-{:04}", track.id);
        format!(
            r#"<event version="2.0" uid="{}" type="{}" how="m-g" time="{}" start="{}" stale="{}"><point lat="{:.7}" lon="{:.7}" hae="{:.2}" ce="{:.2}" le="{:.2}"/><detail><track course="{:.2}" speed="{:.2}"/></detail></event>"#,
            sightline_cot::cot::escape_xml_attr(&uid),
            sightline_cot::cot::escape_xml_attr(cot_type),
            time_str,
            time_str,
            stale_str,
            track.lat_deg,
            track.lon_deg,
            track.hae_m,
            track.ce90_m,
            track.le90_m,
            track.course_deg,
            track.speed_mps,
        )
    };

    let mut relay_loop = RelayLoop::new(
        telem_source,
        det_source.unwrap_or_else(|| {
            UdpDetectionSource::bind("127.0.0.1:0").expect("ephemeral detection source")
        }),
        pipeline,
        queue,
        outbound,
        SystemClock,
        shutdown.clone(),
        limit,
        render,
    );

    let mut total_sent = 0u64;
    while !shutdown.is_triggered() {
        if !relay_loop
            .limit_mut()
            .should_continue(&crate::clock::adapters::RelayClockAdapter(&SystemClock))
        {
            break;
        }

        if let Some(ref mut server) = http_server {
            let _ = server.poll_step();
        }

        match relay_loop.step() {
            Ok(rep) => {
                total_sent += rep.sent as u64;
                if rep.sent > 0 {
                    metrics.record_sent(SystemClock.now_ms());
                }
            }
            Err(e) if crate::error::class::classify(&e).is_fatal() => return Err(e),
            Err(e) => {
                eprintln!("[run_live step error]: {e:?}");
            }
        }
    }

    Ok(total_sent)
}

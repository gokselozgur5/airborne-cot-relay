pub mod app;
pub mod bearing;
pub mod clock;
pub mod config;
pub mod config_file;
pub mod enrich;
pub mod error;
pub mod ingest;
pub mod relay;
pub mod telemetry;

pub use config::{CameraIntrinsics, RelayConfig, RelayError};
pub use relay::{classification_to_cot_type, CotStreamer, TrackSummary};
pub use telemetry::{PlatformPose, TelemetryIngest};
pub mod targeter;
pub use targeter::{GroundIntersection, TargetDetection, Targeter, TrackedObject};
pub mod health;
pub mod http;
pub mod log;
pub mod metrics;
pub mod metrics_exporter;
pub use metrics_exporter::{
    ExporterConfig, MetricFamily, MetricSample, MetricType, PrometheusExporter,
    PROMETHEUS_CONTENT_TYPE,
};
pub mod output;
pub mod pipeline;
pub use pipeline::Pipeline;
pub mod resilience;
pub mod ring_buf;
pub use ring_buf::{AtomicRingBuffer, CotRingBuffer};
pub mod runtime;
pub mod track_filter;
pub mod ttl_cache;
pub use ttl_cache::{CacheEntry, CacheStats, PacketTtlCache, TtlCache, TtlConfig};

pub mod relay_loop;
pub mod xml_sanitize;
pub mod frame_codec;
pub use frame_codec::{Endianness, FrameCodecError, FrameDecoder, LengthField};
pub mod udp_batch;

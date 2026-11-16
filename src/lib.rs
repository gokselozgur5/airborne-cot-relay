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
pub mod fragment_reassembly;
pub use fragment_reassembly::{
    fragment_payload, Fragment, FragmentHeader, FragmentReassembler, ReassemblyError,
    ReassemblyStats, HEADER_SIZE,
};
pub mod runtime;
pub mod track_filter;
pub mod ttl_cache;
pub use ttl_cache::{CacheEntry, CacheStats, PacketTtlCache, TtlCache, TtlConfig};

pub mod relay_loop;
pub mod xml_sanitize;
pub mod frame_codec;
pub use frame_codec::{Endianness, FrameCodecError, FrameDecoder, LengthField};
pub mod udp_batch;
pub mod json_cot_codec;
pub use json_cot_codec::{
    cot_xml_to_json, json_to_cot_xml, CotEventDto, CotToJsonStream, JsonCotError, JsonToCotStream,
};
pub mod multicast_rx;
pub mod bloom_dedup;
pub mod rate_limiter;
pub use rate_limiter::{RateLimiterConfig, RateLimiterStats, TokenBucket};
pub mod telemetry_snapshot;
pub use telemetry_snapshot::{DualBufferSampler, SamplerStats, TelemetrySnapshot};
pub mod reconnect_client;
pub use reconnect_client::{
    ReconnectClient, ReconnectClientError, ReconnectPolicy, StreamClientStats,
};
pub mod schema_version;
pub use schema_version::{
    downgrade_cot_xml, CotSchemaVersion, DowngradeTransformer, SchemaNegotiator,
    SchemaVersionError,
};
pub mod priority_scheduler;

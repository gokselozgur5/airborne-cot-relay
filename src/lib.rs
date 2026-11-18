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

pub mod frame_codec;
pub mod relay_loop;
pub mod xml_sanitize;
pub use frame_codec::{Endianness, FrameCodecError, FrameDecoder, LengthField};
pub mod json_cot_codec;
pub mod udp_batch;
pub use json_cot_codec::{
    cot_xml_to_json, json_to_cot_xml, CotEventDto, CotToJsonStream, JsonCotError, JsonToCotStream,
};
pub mod bloom_dedup;
pub mod multicast_rx;
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
    downgrade_cot_xml, CotSchemaVersion, DowngradeTransformer, SchemaNegotiator, SchemaVersionError,
};
pub mod priority_scheduler;
pub use priority_scheduler::{
    CotPriority, PrioritizedPacket, PriorityPacketScheduler, SchedulerError, SchedulerStats,
};
pub mod retransmit_tracker;
pub use retransmit_tracker::{
    PacketRecord, RbtConfig, RetransmitTracker, TrackerError, TrackerStats,
};
pub mod multicast_listener;
pub use multicast_listener::{
    DropPolicy, DroppingStats, DroppingStatsSnapshot, MulticastListener, MulticastListenerConfig,
    MulticastListenerError, MulticastPacket,
};
pub mod heartbeat;
pub use heartbeat::{
    HeartbeatConfig, HeartbeatGenerator, HeartbeatRecord, PlatformMetrics,
    PlatformOperationalState, RadioMetrics,
};
pub mod dedup_cache;
pub use dedup_cache::{CotDedupCache, DedupBloomFilter, DedupCacheConfig, DedupStats};

pub mod payload_compressor;
pub use payload_compressor::{
    CompressedPayload, CompressionAlgorithm, CompressorConfig, CompressorError, CompressorStats,
    CotPayloadCompressor,
};

pub mod bandwidth_throttle;
pub use bandwidth_throttle::{
    BandwidthThrottle, CongestionState, DropReason, ThrottleConfig, ThrottleDecision, ThrottleStats,
};

pub mod cot_signer;
pub use cot_signer::{bytes_to_hex, constant_time_compare, hmac_sha256, sha256};
pub use cot_signer::{
    CotSignatureValidator, CotSigner, CotSignerConfig, CotSignerError, CotSignerStats,
    CotValidatorStats, SignedCotFrame, COT_HEADER_LEN, COT_SIG_MAGIC, COT_SIG_VERSION,
};

pub mod ring_recorder;
pub use ring_recorder::{
    CrashDumpError, CrashDumpReport, DumpFormat, OverwriteMode, RecorderStats, RingRecorderConfig,
    TelemetryRecord, TelemetryRingRecorder,
};

pub mod failover_router;
pub use failover_router::{
    FailoverPolicy, FailoverRouter, NetworkInterface, RouteKind, RouterConfig, RouterError,
    RouterStats, RoutingDecision,
};

pub mod fragmentation;
pub use fragmentation::{
    crc16_ccitt, FragmentHeader as CotFragmentHeader, FragmentPacket, FragmentationError,
    FragmentationStats, FragmenterConfig, InFlightSession, PayloadFragmenter, ReassemblyConfig,
    ReassemblyEngine, DEFAULT_MTU, HEADER_SIZE as FRAG_HEADER_SIZE, MAGIC_BYTES, MIN_MTU,
    PROTOCOL_VERSION,
};

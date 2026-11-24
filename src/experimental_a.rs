#[path = "bandwidth_throttle.rs"]
pub mod bandwidth_throttle;
pub use bandwidth_throttle::{
    BandwidthThrottle, CongestionState, DropReason, ThrottleConfig, ThrottleDecision, ThrottleStats,
};

#[path = "bloom_dedup.rs"]
pub mod bloom_dedup;

#[path = "cot_signer.rs"]
pub mod cot_signer;
pub use cot_signer::{bytes_to_hex, constant_time_compare, hmac_sha256, sha256};
pub use cot_signer::{
    CotSignatureValidator, CotSigner, CotSignerConfig, CotSignerError, CotSignerStats,
    CotValidatorStats, SignedCotFrame, COT_HEADER_LEN, COT_SIG_MAGIC, COT_SIG_VERSION,
};

#[path = "dedup_cache.rs"]
pub mod dedup_cache;
pub use dedup_cache::{CotDedupCache, DedupBloomFilter, DedupCacheConfig, DedupStats};

#[path = "failover_router.rs"]
pub mod failover_router;
pub use failover_router::{
    FailoverPolicy, FailoverRouter, NetworkInterface, RouteKind, RouterConfig, RouterError,
    RouterStats, RoutingDecision,
};

#[path = "fragment_reassembly.rs"]
pub mod fragment_reassembly;

#[path = "fragmentation.rs"]
pub mod fragmentation;
pub use fragmentation::{
    crc16_ccitt, FragmentHeader, FragmentPacket, FragmentationError, FragmentationStats,
    FragmenterConfig, InFlightSession, PayloadFragmenter, ReassemblyConfig, ReassemblyEngine,
    DEFAULT_MTU, HEADER_SIZE, MAGIC_BYTES, MIN_MTU, PROTOCOL_VERSION,
};

#[path = "frame_codec.rs"]
pub mod frame_codec;
pub use frame_codec::{Endianness, FrameCodecError, FrameDecoder, LengthField};

#[path = "heartbeat.rs"]
pub mod heartbeat;
pub use heartbeat::{
    HeartbeatConfig, HeartbeatGenerator, HeartbeatRecord, PlatformMetrics,
    PlatformOperationalState, RadioMetrics,
};

#[path = "json_cot_codec.rs"]
pub mod json_cot_codec;
pub use json_cot_codec::{
    cot_xml_to_json, json_to_cot_xml, CotEventDto, CotToJsonStream, JsonCotError, JsonToCotStream,
};

#[path = "metrics_exporter.rs"]
pub mod metrics_exporter;
pub use metrics_exporter::{
    ExporterConfig, MetricFamily, MetricSample, MetricType, PrometheusExporter,
    PROMETHEUS_CONTENT_TYPE,
};

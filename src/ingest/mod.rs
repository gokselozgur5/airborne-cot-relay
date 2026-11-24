//! Input sources for telemetry and detections over network protocols.

pub mod cobs;
pub mod json;
pub mod json_udp;
pub mod mavlink;
pub mod mavlink_udp;
pub mod serial_cobs;
pub mod source_filter;
pub mod tcp_telemetry;
pub mod udp_detections;
pub mod udp_telemetry;

pub use json_udp::{parse_json_detection, JsonUdpDetectionSource};
pub use mavlink_udp::{
    decode_mavlink_v2_frame, MavlinkMessage, MavlinkPacket, MavlinkPoseSynthesizer,
    MavlinkUdpSource,
};
pub use serial_cobs::{cobs_decode, cobs_encode, SerialCobsReader};
pub use source_filter::SourceFilter;
pub use tcp_telemetry::TcpTelemetrySource;
pub use udp_detections::UdpDetectionSource;
pub use udp_telemetry::UdpTelemetrySource;

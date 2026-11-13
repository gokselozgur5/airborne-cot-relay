//! Input sources for telemetry and detections over network protocols.

pub mod udp_detections;
pub mod udp_telemetry;

pub use udp_detections::UdpDetectionSource;
pub use udp_telemetry::UdpTelemetrySource;

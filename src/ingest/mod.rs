//! Input sources for telemetry and detections over network protocols.

pub mod tcp_telemetry;
pub mod udp_detections;
pub mod udp_telemetry;

pub use tcp_telemetry::TcpTelemetrySource;
pub use udp_detections::UdpDetectionSource;
pub use udp_telemetry::UdpTelemetrySource;

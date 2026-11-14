//! Maps `[input.NAME]` sections to input source specifications.

use crate::config::RelayError;
use std::net::SocketAddr;

/// Supported kinds of ingest sources.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputKind {
    UdpTelemetry,
    TcpTelemetry,
    UdpDetections,
}

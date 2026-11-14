//! Source IP allowlist filtering for telemetry datagrams.

use std::collections::HashSet;
use std::net::IpAddr;

/// Filters incoming datagrams against an allowlist of IP addresses.
#[derive(Debug, Clone, Default)]
pub struct SourceFilter {
    allowed: HashSet<IpAddr>,
    rejected: u64,
}

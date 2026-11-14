//! Source IP allowlist filtering for telemetry datagrams.

use std::collections::HashSet;
use std::net::IpAddr;

/// Filters incoming datagrams against an allowlist of IP addresses.
#[derive(Debug, Clone, Default)]
pub struct SourceFilter {
    allowed: HashSet<IpAddr>,
    rejected: u64,
}

impl SourceFilter {
    /// Creates a filter with the given allowed IP addresses.
    pub fn new(allowed: impl IntoIterator<Item = IpAddr>) -> Self {
        Self {
            allowed: allowed.into_iter().collect(),
            rejected: 0,
        }
    }

    /// Number of datagrams rejected so far.
    pub fn rejected_count(&self) -> u64 {
        self.rejected
    }
}

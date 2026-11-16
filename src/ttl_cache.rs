//! In-memory TTL packet cache with deterministic time expiry cleaning.
//!
//! Provides time-to-live caching for telemetry, CoT events, and network packets
//! with configurable eviction policies and expired item reaping.

/// Configuration options for TTL cache behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TtlConfig {
    /// Time-to-live duration in milliseconds.
    pub ttl_ms: u64,
    /// Maximum number of items before oldest entries are evicted.
    pub max_capacity: usize,
}

impl Default for TtlConfig {
    fn default() -> Self {
        Self {
            ttl_ms: 30_000,
            max_capacity: 10_000,
        }
    }
}

impl TtlConfig {
    /// Creates a configuration with explicit TTL and max capacity.
    pub const fn new(ttl_ms: u64, max_capacity: usize) -> Self {
        Self {
            ttl_ms,
            max_capacity,
        }
    }
}

/// Statistics on cache operations.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CacheStats {
    /// Number of successful lookups.
    pub hits: u64,
    /// Number of lookups finding no entry or expired entry.
    pub misses: u64,
    /// Number of entries evicted due to capacity limits.
    pub evictions: u64,
    /// Number of entries dropped due to TTL expiry.
    pub expirations: u64,
}

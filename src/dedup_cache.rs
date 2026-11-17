//! CoT packet deduplication cache with sliding expiration and bloom filter.

use std::collections::{HashMap, VecDeque};

/// Configuration for the deduplication cache.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DedupCacheConfig {
    /// Duration in milliseconds for sliding window expiration.
    pub sliding_window_ms: u64,
    /// Maximum entries stored in exact cache before eviction.
    pub max_capacity: usize,
    /// Number of bits in the backing Bloom filter.
    pub bloom_bits: usize,
    /// Number of hash functions for the Bloom filter.
    pub bloom_hashes: usize,
}

impl Default for DedupCacheConfig {
    fn default() -> Self {
        Self {
            sliding_window_ms: 10_000,
            max_capacity: 50_000,
            bloom_bits: 65_536,
            bloom_hashes: 4,
        }
    }
}

/// Statistics on deduplication operations.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DedupStats {
    /// Total packets checked.
    pub total_checked: u64,
    /// Duplicates detected and dropped.
    pub duplicates_dropped: u64,
    /// Unique packets accepted.
    pub unique_passed: u64,
    /// Exact entries expired.
    pub expired_count: u64,
    /// Entries evicted due to capacity.
    pub evictions: u64,
}

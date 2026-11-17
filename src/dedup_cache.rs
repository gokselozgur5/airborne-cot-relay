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

#[derive(Debug, Clone, PartialEq, Eq)]
struct SlidingEntry {
    last_seen_ms: u64,
    expires_at_ms: u64,
}

impl SlidingEntry {
    fn new(now_ms: u64, window_ms: u64) -> Self {
        Self {
            last_seen_ms: now_ms,
            expires_at_ms: now_ms.saturating_add(window_ms),
        }
    }

    fn refresh(&mut self, now_ms: u64, window_ms: u64) {
        self.last_seen_ms = now_ms;
        self.expires_at_ms = now_ms.saturating_add(window_ms);
    }

    fn is_expired(&self, now_ms: u64) -> bool {
        now_ms >= self.expires_at_ms
    }
}

/// Compact bloom filter for fast duplicate screening.
#[derive(Debug, Clone)]
pub struct DedupBloomFilter {
    bits: Vec<u64>,
    num_bits: usize,
    num_hashes: usize,
}

impl DedupBloomFilter {
    /// Creates a new bloom filter with given bit capacity and hash count.
    pub fn new(num_bits: usize, num_hashes: usize) -> Self {
        let num_bits = num_bits.max(64);
        let num_words = (num_bits + 63) / 64;
        let num_hashes = num_hashes.clamp(1, 16);
        Self {
            bits: vec![0u64; num_words],
            num_bits,
            num_hashes,
        }
    }
    fn hash_pair(data: &[u8]) -> (u64, u64) {
        let mut h1 = 0xcbf29ce484222325u64;
        for &b in data {
            h1 = (h1 ^ (b as u64)).wrapping_mul(0x100000001b3u64);
        }
        let mut h2 = 0x517cc1b727220a95u64;
        for &b in data {
            h2 = (h2 ^ (b as u64)).wrapping_mul(0x100000001b3u64);
        }
        (h1, if h2 == 0 { 1 } else { h2 })
    }

    /// Inserts raw byte key into bloom filter.
    pub fn insert(&mut self, data: &[u8]) {
        let (h1, h2) = Self::hash_pair(data);
        for i in 0..self.num_hashes {
            let comb = h1.wrapping_add((i as u64).wrapping_mul(h2));
            let bit_idx = (comb % self.num_bits as u64) as usize;
            self.bits[bit_idx / 64] |= 1u64 << (bit_idx % 64);
        }
    }

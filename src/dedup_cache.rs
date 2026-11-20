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
        let num_words = num_bits.div_ceil(64);
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
    /// Checks whether key may be present in the bloom filter.
    pub fn contains(&self, data: &[u8]) -> bool {
        let (h1, h2) = Self::hash_pair(data);
        for i in 0..self.num_hashes {
            let comb = h1.wrapping_add((i as u64).wrapping_mul(h2));
            let bit_idx = (comb % self.num_bits as u64) as usize;
            if (self.bits[bit_idx / 64] & (1u64 << (bit_idx % 64))) == 0 {
                return false;
            }
        }
        true
    }

    /// Clears all bits in bloom filter.
    pub fn clear(&mut self) {
        self.bits.fill(0);
    }

    /// Returns count of set bits in bloom filter.
    pub fn count_ones(&self) -> usize {
        self.bits.iter().map(|w| w.count_ones() as usize).sum()
    }
}

/// Primary deduplication cache combining sliding window expiration and Bloom filtering.
#[derive(Debug, Clone)]
pub struct CotDedupCache {
    config: DedupCacheConfig,
    bloom: DedupBloomFilter,
    entries: HashMap<Vec<u8>, SlidingEntry>,
    expiry_queue: VecDeque<(u64, Vec<u8>)>,
    stats: DedupStats,
}

impl CotDedupCache {
    /// Creates a new deduplication cache with default configuration.
    pub fn new() -> Self {
        Self::with_config(DedupCacheConfig::default())
    }

    /// Creates a new deduplication cache with custom configuration.
    pub fn with_config(config: DedupCacheConfig) -> Self {
        let bloom = DedupBloomFilter::new(config.bloom_bits, config.bloom_hashes);
        Self {
            config,
            bloom,
            entries: HashMap::new(),
            expiry_queue: VecDeque::new(),
            stats: DedupStats::default(),
        }
    }
    /// Returns number of items currently in cache.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if cache contains no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns operational statistics.
    pub fn stats(&self) -> DedupStats {
        self.stats
    }

    /// Returns reference to internal bloom filter.
    pub fn bloom(&self) -> &DedupBloomFilter {
        &self.bloom
    }

    /// Returns the number of entries waiting in the expiry queue.
    pub fn expiry_queue_len(&self) -> usize {
        self.expiry_queue.len()
    }
    fn evict_if_needed(&mut self) {
        while self.entries.len() >= self.config.max_capacity {
            let Some((queued_exp, oldest_key)) = self.expiry_queue.pop_front() else {
                break;
            };
            if let Some(entry) = self.entries.get(&oldest_key) {
                if entry.expires_at_ms == queued_exp {
                    self.entries.remove(&oldest_key);
                    self.stats.evictions += 1;
                    break;
                }
            }
        }
    }

    /// Clears all entries, queues, and resets bloom filter.
    pub fn clear(&mut self) {
        self.bloom.clear();
        self.entries.clear();
        self.expiry_queue.clear();
    }
    /// Prunes expired entries based on current millisecond timestamp.
    pub fn prune_expired(&mut self, now_ms: u64) -> usize {
        let mut pruned = 0;
        while let Some((expires_at, _)) = self.expiry_queue.front() {
            if *expires_at <= now_ms {
                let (_, key) = self.expiry_queue.pop_front().unwrap();
                if let Some(entry) = self.entries.get(&key) {
                    if entry.is_expired(now_ms) {
                        self.entries.remove(&key);
                        pruned += 1;
                    }
                }
            } else {
                break;
            }
        }
        self.stats.expired_count += pruned as u64;
        if self.entries.is_empty() {
            self.bloom.clear();
        }
        pruned
    }
    /// Checks packet key against cache; records entry and returns true if unique.
    pub fn check_and_insert(&mut self, key: &[u8], now_ms: u64) -> bool {
        self.stats.total_checked += 1;
        self.prune_expired(now_ms);

        if self.bloom.contains(key) {
            if let Some(entry) = self.entries.get_mut(key) {
                if !entry.is_expired(now_ms) {
                    entry.refresh(now_ms, self.config.sliding_window_ms);
                    self.stats.duplicates_dropped += 1;
                    return false;
                }
            }
        }

        self.evict_if_needed();
        let entry = SlidingEntry::new(now_ms, self.config.sliding_window_ms);
        let expires_at = entry.expires_at_ms;
        self.bloom.insert(key);
        self.entries.insert(key.to_vec(), entry);
        self.expiry_queue.push_back((expires_at, key.to_vec()));
        self.stats.unique_passed += 1;
        true
    }
    /// Checks whether key is duplicate without mutating entry or sliding timestamp.
    pub fn is_duplicate(&self, key: &[u8], now_ms: u64) -> bool {
        if !self.bloom.contains(key) {
            return false;
        }
        self.entries.get(key).is_some_and(|e| !e.is_expired(now_ms))
    }

    /// Convenience wrapper for string keys.
    pub fn check_and_insert_str(&mut self, key: &str, now_ms: u64) -> bool {
        self.check_and_insert(key.as_bytes(), now_ms)
    }

    /// Convenience helper for CoT UID and millisecond timestamp combinations.
    pub fn check_and_insert_uid_time(&mut self, uid: &str, time_ms: u64, now_ms: u64) -> bool {
        let mut key = Vec::with_capacity(uid.len() + 9);
        key.extend_from_slice(uid.as_bytes());
        key.push(b':');
        key.extend_from_slice(&time_ms.to_be_bytes());
        self.check_and_insert(&key, now_ms)
    }
}

impl Default for CotDedupCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bloom_filter_basic() {
        let mut bloom = DedupBloomFilter::new(512, 4);
        assert_eq!(bloom.count_ones(), 0);
        assert!(!bloom.contains(b"alpha"));
        bloom.insert(b"alpha");
        assert!(bloom.contains(b"alpha"));
        assert!(!bloom.contains(b"beta"));
        bloom.clear();
        assert_eq!(bloom.count_ones(), 0);
        assert!(!bloom.contains(b"alpha"));
    }
    #[test]
    fn test_cache_insert_and_duplicate() {
        let mut cache = CotDedupCache::new();
        assert!(cache.check_and_insert(b"cot-packet-1", 1000));
        assert_eq!(cache.len(), 1);
        assert!(!cache.check_and_insert(b"cot-packet-1", 1500));
        assert_eq!(cache.stats().duplicates_dropped, 1);
        assert_eq!(cache.stats().unique_passed, 1);
        assert!(cache.is_duplicate(b"cot-packet-1", 1600));
    }
    #[test]
    fn test_cache_sliding_expiration() {
        let config = DedupCacheConfig {
            sliding_window_ms: 1000,
            ..Default::default()
        };
        let mut cache = CotDedupCache::with_config(config);
        assert!(cache.check_and_insert(b"p1", 1000));
        assert!(!cache.check_and_insert(b"p1", 1500));
        assert!(!cache.check_and_insert(b"p1", 2400));
        assert!(cache.check_and_insert(b"p1", 3500));
        assert_eq!(cache.stats().unique_passed, 2);
    }
    #[test]
    fn test_cache_capacity_eviction_and_uid_time() {
        let config = DedupCacheConfig {
            sliding_window_ms: 10_000,
            max_capacity: 2,
            bloom_bits: 512,
            bloom_hashes: 3,
        };
        let mut cache = CotDedupCache::with_config(config);
        assert!(cache.check_and_insert_uid_time("uav-1", 100, 1000));
        assert!(cache.check_and_insert_uid_time("uav-2", 200, 1000));
        assert!(cache.check_and_insert_uid_time("uav-3", 300, 1000));
        assert_eq!(cache.stats().evictions, 1);
        assert_eq!(cache.len(), 2);
    }
}

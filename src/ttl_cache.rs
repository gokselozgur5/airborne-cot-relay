//! In-memory TTL packet cache with deterministic time expiry cleaning.
//!
//! Provides time-to-live caching for telemetry, CoT events, and network packets
//! with configurable eviction policies and expired item reaping.

use std::collections::HashMap;
use std::hash::Hash;

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

/// An entry stored in the cache with timing metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheEntry<V> {
    /// Cached value payload.
    pub value: V,
    /// Millisecond timestamp when entry was inserted.
    pub inserted_at_ms: u64,
    /// Millisecond timestamp when entry expires.
    pub expires_at_ms: u64,
}

impl<V> CacheEntry<V> {
    /// Creates a new cache entry with calculated expiration.
    pub fn new(value: V, inserted_at_ms: u64, ttl_ms: u64) -> Self {
        Self {
            value,
            inserted_at_ms,
            expires_at_ms: inserted_at_ms.saturating_add(ttl_ms),
        }
    }

    /// Checks if this entry is expired at the given timestamp.
    pub fn is_expired(&self, now_ms: u64) -> bool {
        now_ms >= self.expires_at_ms
    }
}

/// Cache with time-to-live expiration and periodic cleaning.
#[derive(Debug, Clone)]
pub struct TtlCache<K, V> {
    entries: HashMap<K, CacheEntry<V>>,
    config: TtlConfig,
    stats: CacheStats,
}

impl<K, V> TtlCache<K, V>
where
    K: Eq + Hash + Clone,
{
    /// Creates a new cache with the specified configuration.
    pub fn new(config: TtlConfig) -> Self {
        Self {
            entries: HashMap::new(),
            config,
            stats: CacheStats::default(),
        }
    }

    /// Creates a new cache with default capacity and given TTL.
    pub fn with_ttl(ttl_ms: u64) -> Self {
        Self::new(TtlConfig {
            ttl_ms,
            ..TtlConfig::default()
        })
    }

    /// Returns the number of stored entries (including uncollected expired ones).
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if the cache contains no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns configured maximum capacity.
    pub fn capacity(&self) -> usize {
        self.config.max_capacity
    }

    /// Returns current cache operation statistics.
    pub fn stats(&self) -> CacheStats {
        self.stats
    }

    /// Evicts all expired entries as of now_ms and returns count removed.
    pub fn clean_expired(&mut self, now_ms: u64) -> usize {
        let before = self.entries.len();
        self.entries.retain(|_, entry| !entry.is_expired(now_ms));
        let evicted = before.saturating_sub(self.entries.len());
        self.stats.expirations = self.stats.expirations.saturating_add(evicted as u64);
        evicted
    }

    /// Evicts the single oldest entry by insertion timestamp.
    fn evict_oldest(&mut self) {
        if let Some(oldest_key) = self
            .entries
            .iter()
            .min_by_key(|(_, entry)| entry.inserted_at_ms)
            .map(|(k, _)| k.clone())
        {
            self.entries.remove(&oldest_key);
            self.stats.evictions = self.stats.evictions.saturating_add(1);
        }
    }

    /// Inserts a key-value pair, evicting expired or oldest items if at capacity.
    pub fn insert(&mut self, key: K, value: V, now_ms: u64) -> Option<V> {
        if self.entries.len() >= self.config.max_capacity && !self.entries.contains_key(&key) {
            self.clean_expired(now_ms);
            if self.entries.len() >= self.config.max_capacity {
                self.evict_oldest();
            }
        }
        let entry = CacheEntry::new(value, now_ms, self.config.ttl_ms);
        self.entries.insert(key, entry).map(|old| old.value)
    }

    /// Returns a reference to the value if present and unexpired.
    pub fn get(&mut self, key: &K, now_ms: u64) -> Option<&V> {
        if let Some(entry) = self.entries.get(key) {
            if entry.is_expired(now_ms) {
                self.entries.remove(key);
                self.stats.expirations = self.stats.expirations.saturating_add(1);
                self.stats.misses = self.stats.misses.saturating_add(1);
                None
            } else {
                self.stats.hits = self.stats.hits.saturating_add(1);
                self.entries.get(key).map(|e| &e.value)
            }
        } else {
            self.stats.misses = self.stats.misses.saturating_add(1);
            None
        }
    }

    /// Peeks at a value without modifying hit/miss statistics.
    pub fn peek(&self, key: &K, now_ms: u64) -> Option<&V> {
        self.entries
            .get(key)
            .filter(|entry| !entry.is_expired(now_ms))
            .map(|entry| &entry.value)
    }

    /// Checks if an unexpired key exists in the cache.
    pub fn contains_key(&self, key: &K, now_ms: u64) -> bool {
        self.peek(key, now_ms).is_some()
    }

    /// Removes an entry by key regardless of expiration state.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.entries.remove(key).map(|entry| entry.value)
    }

    /// Clears all entries from the cache.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Reaps and returns all expired (key, value) pairs as of now_ms.
    pub fn drain_expired(&mut self, now_ms: u64) -> Vec<(K, V)> {
        let mut expired_keys = Vec::new();
        for (k, entry) in &self.entries {
            if entry.is_expired(now_ms) {
                expired_keys.push(k.clone());
            }
        }
        let mut reaped = Vec::with_capacity(expired_keys.len());
        for k in expired_keys {
            if let Some(entry) = self.entries.remove(&k) {
                reaped.push((k, entry.value));
            }
        }
        self.stats.expirations = self.stats.expirations.saturating_add(reaped.len() as u64);
        reaped
    }
}

/// Convenience alias for byte packet caching by packet UID or sequence ID.
pub type PacketTtlCache = TtlCache<String, Vec<u8>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get_basic() {
        let mut cache = TtlCache::with_ttl(1_000);
        assert!(cache.is_empty());
        assert_eq!(cache.insert("key1", 42, 100), None);
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.get(&"key1", 150), Some(&42));
        assert_eq!(cache.stats().hits, 1);
        assert_eq!(cache.get(&"missing", 150), None);
        assert_eq!(cache.stats().misses, 1);
    }

    #[test]
    fn test_ttl_expiry_and_contains() {
        let mut cache = TtlCache::with_ttl(500);
        cache.insert("p1", 100, 1_000);
        assert!(cache.contains_key(&"p1", 1_400));
        assert_eq!(cache.peek(&"p1", 1_400), Some(&100));
        assert!(!cache.contains_key(&"p1", 1_500));
        assert_eq!(cache.get(&"p1", 1_501), None);
        assert_eq!(cache.stats().expirations, 1);
        assert_eq!(cache.stats().misses, 1);
    }

    #[test]
    fn test_clean_expired_batch() {
        let mut cache = TtlCache::with_ttl(200);
        cache.insert("a", 1, 100);
        cache.insert("b", 2, 150);
        cache.insert("c", 3, 250);
        assert_eq!(cache.clean_expired(360), 2);
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.get(&"c", 360), Some(&3));
        assert_eq!(cache.stats().expirations, 2);
    }

    #[test]
    fn test_capacity_eviction_oldest() {
        let config = TtlConfig::new(10_000, 2);
        let mut cache = TtlCache::new(config);
        cache.insert("first", 1, 100);
        cache.insert("second", 2, 200);
        assert_eq!(cache.len(), 2);
        cache.insert("third", 3, 300);
        assert_eq!(cache.len(), 2);
        assert_eq!(cache.stats().evictions, 1);
        assert_eq!(cache.peek(&"first", 300), None);
        assert_eq!(cache.peek(&"second", 300), Some(&2));
        assert_eq!(cache.peek(&"third", 300), Some(&3));
    }

    #[test]
    fn test_drain_expired_and_packet_cache() {
        let mut cache: PacketTtlCache = TtlCache::with_ttl(100);
        cache.insert("cot-001".to_string(), vec![1, 2, 3], 1_000);
        cache.insert("cot-002".to_string(), vec![4, 5, 6], 1_050);
        cache.insert("cot-003".to_string(), vec![7, 8, 9], 1_150);

        let reaped = cache.drain_expired(1_160);
        assert_eq!(reaped.len(), 2);
        assert_eq!(cache.len(), 1);
        assert_eq!(
            cache.peek(&"cot-003".to_string(), 1_160),
            Some(&vec![7, 8, 9])
        );
    }
}

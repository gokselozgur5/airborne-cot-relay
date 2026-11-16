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
}

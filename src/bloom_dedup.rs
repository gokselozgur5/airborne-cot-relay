//! Cursor-on-Target telemetry deduplication Bloom filter.

/// Computes a 64-bit FNV-1a hash of the given byte slice.
pub fn fnv1a_hash(data: &[u8], seed: u64) -> u64 {
    let mut hash = 0xcbf29ce484222325u64 ^ seed;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3u64);
    }
    hash
}

/// Generates two independent 64-bit hashes for Kirsch-Mitzenmacher double-hashing.
pub fn double_hash(data: &[u8]) -> (u64, u64) {
    let h1 = fnv1a_hash(data, 0);
    let h2 = fnv1a_hash(data, 0x517cc1b727220a95);
    (h1, if h2 == 0 { 1 } else { h2 })
}

/// Probabilistic bitset set for efficient membership testing.
#[derive(Debug, Clone)]
pub struct BloomFilter {
    bits: Vec<u64>,
    num_bits: usize,
    num_hashes: usize,
}

impl BloomFilter {
    /// Creates a new BloomFilter with specified bit capacity and hash count.
    pub fn new(num_bits: usize, num_hashes: usize) -> Self {
        let num_bits = num_bits.max(64);
        let num_words = (num_bits + 63) / 64;
        let num_hashes = num_hashes.max(1).min(16);
        Self {
            bits: vec![0u64; num_words],
            num_bits,
            num_hashes,
        }
    }

    /// Inserts raw byte slice into the filter.
    pub fn insert(&mut self, data: &[u8]) {
        let (h1, h2) = double_hash(data);
        for i in 0..self.num_hashes {
            let combined = h1.wrapping_add((i as u64).wrapping_mul(h2));
            let bit_idx = (combined % self.num_bits as u64) as usize;
            self.bits[bit_idx / 64] |= 1u64 << (bit_idx % 64);
        }
    }

    /// Checks whether the filter may contain the byte slice.
    pub fn contains(&self, data: &[u8]) -> bool {
        let (h1, h2) = double_hash(data);
        for i in 0..self.num_hashes {
            let combined = h1.wrapping_add((i as u64).wrapping_mul(h2));
            let bit_idx = (combined % self.num_bits as u64) as usize;
            if (self.bits[bit_idx / 64] & (1u64 << (bit_idx % 64))) == 0 {
                return false;
            }
        }
        true
    }

    /// Clears all bits in the filter.
    pub fn clear(&mut self) {
        self.bits.fill(0);
    }

    /// Returns the number of set bits (popcount) in the filter.
    pub fn count_ones(&self) -> usize {
        self.bits.iter().map(|word| word.count_ones() as usize).sum()
    }

    /// Returns true if no bits are set.
    pub fn is_empty(&self) -> bool {
        self.bits.iter().all(|&word| word == 0)
    }
}

/// Deduplicator for Cursor-on-Target telemetry events using a Bloom filter.
#[derive(Debug, Clone)]
pub struct CotBloomDeduplicator {
    filter: BloomFilter,
    total_checked: u64,
    duplicates_dropped: u64,
    unique_passed: u64,
}

impl CotBloomDeduplicator {
    /// Creates a deduplicator with specified filter capacity and hash function count.
    pub fn new(capacity_bits: usize, hash_count: usize) -> Self {
        Self {
            filter: BloomFilter::new(capacity_bits, hash_count),
            total_checked: 0,
            duplicates_dropped: 0,
            unique_passed: 0,
        }
    }

    /// Checks a raw byte slice. Returns true if unique (new), false if duplicate.
    pub fn check_and_record_raw(&mut self, data: &[u8]) -> bool {
        self.total_checked += 1;
        if self.filter.contains(data) {
            self.duplicates_dropped += 1;
            false
        } else {
            self.filter.insert(data);
            self.unique_passed += 1;
            true
        }
    }

    /// Checks a CoT (UID, time) tuple. Returns true if unique (new), false if duplicate.
    pub fn check_and_record_uid_time(&mut self, uid: &str, time_str: &str) -> bool {
        let mut key = Vec::with_capacity(uid.len() + time_str.len() + 1);
        key.extend_from_slice(uid.as_bytes());
        key.push(b':');
        key.extend_from_slice(time_str.as_bytes());
        self.check_and_record_raw(&key)
    }

    /// Resets the internal Bloom filter for a new telemetry epoch.
    pub fn reset_window(&mut self) {
        self.filter.clear();
    }

    /// Total packets checked since creation.
    pub fn total_checked(&self) -> u64 {
        self.total_checked
    }

    /// Number of duplicate packets dropped.
    pub fn duplicates_dropped(&self) -> u64 {
        self.duplicates_dropped
    }

    /// Number of unique packets passed through.
    pub fn unique_passed(&self) -> u64 {
        self.unique_passed
    }
}

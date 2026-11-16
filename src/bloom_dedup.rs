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
}

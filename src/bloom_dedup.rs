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

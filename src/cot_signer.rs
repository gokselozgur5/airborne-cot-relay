//! Cryptographic HMAC-SHA256 signature generator and validator for secure CoT.
//!
//! Pure-Rust FIPS 180-4 SHA-256 and RFC 2104 HMAC-SHA256 implementation
//! with wire framing, anti-replay skew checks, and constant-time verify.

use std::collections::HashMap;
use std::fmt;

const K256: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

const H256_INIT: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

#[inline]
fn rotr(x: u32, n: u32) -> u32 {
    (x >> n) | (x << (32 - n))
}

#[inline]
fn ch(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (!x & z)
}

#[inline]
fn maj(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (x & z) ^ (y & z)
}

#[inline]
fn sigma0(x: u32) -> u32 {
    rotr(x, 2) ^ rotr(x, 13) ^ rotr(x, 22)
}

#[inline]
fn sigma1(x: u32) -> u32 {
    rotr(x, 6) ^ rotr(x, 11) ^ rotr(x, 25)
}

#[inline]
fn gamma0(x: u32) -> u32 {
    rotr(x, 7) ^ rotr(x, 18) ^ (x >> 3)
}

#[inline]
fn gamma1(x: u32) -> u32 {
    rotr(x, 17) ^ rotr(x, 19) ^ (x >> 10)
}

fn sha256_block(state: &mut [u32; 8], block: &[u8; 64]) {
    let mut w = [0u32; 64];
    for t in 0..16 {
        w[t] = u32::from_be_bytes([
            block[t * 4],
            block[t * 4 + 1],
            block[t * 4 + 2],
            block[t * 4 + 3],
        ]);
    }
    for t in 16..64 {
        w[t] = gamma1(w[t - 2])
            .wrapping_add(w[t - 7])
            .wrapping_add(gamma0(w[t - 15]))
            .wrapping_add(w[t - 16]);
    }
    let mut a = state[0];
    let mut b = state[1];
    let mut c = state[2];
    let mut d = state[3];
    let mut e = state[4];
    let mut f = state[5];
    let mut g = state[6];
    let mut h = state[7];

    for t in 0..64 {
        let t1 = h
            .wrapping_add(sigma1(e))
            .wrapping_add(ch(e, f, g))
            .wrapping_add(K256[t])
            .wrapping_add(w[t]);
        let t2 = sigma0(a).wrapping_add(maj(a, b, c));
        h = g;
        g = f;
        f = e;
        e = d.wrapping_add(t1);
        d = c;
        c = b;
        b = a;
        a = t1.wrapping_add(t2);
    }
    state[0] = state[0].wrapping_add(a);
    state[1] = state[1].wrapping_add(b);
    state[2] = state[2].wrapping_add(c);
    state[3] = state[3].wrapping_add(d);
    state[4] = state[4].wrapping_add(e);
    state[5] = state[5].wrapping_add(f);
    state[6] = state[6].wrapping_add(g);
    state[7] = state[7].wrapping_add(h);
}

/// Compute FIPS 180-4 SHA-256 256-bit cryptographic hash.
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut state = H256_INIT;
    let bit_len = (data.len() as u64).wrapping_mul(8);
    let mut offset = 0;

    while offset + 64 <= data.len() {
        let mut block = [0u8; 64];
        block.copy_from_slice(&data[offset..offset + 64]);
        sha256_block(&mut state, &block);
        offset += 64;
    }
    let rem = &data[offset..];
    let mut buf = Vec::with_capacity(128);
    buf.extend_from_slice(rem);
    buf.push(0x80);
    while (buf.len() + 8) % 64 != 0 {
        buf.push(0x00);
    }
    buf.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in buf.chunks_exact(64) {
        let mut block = [0u8; 64];
        block.copy_from_slice(chunk);
        sha256_block(&mut state, &block);
    }
    let mut out = [0u8; 32];
    for (i, word) in state.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}

/// Constant-time slice equality comparison against timing side channels.
pub fn constant_time_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Format byte slice as lowercase hexadecimal string.
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}

/// Compute RFC 2104 HMAC-SHA256 message authentication code.
pub fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    let mut k_pad = [0u8; 64];
    if key.len() > 64 {
        let kh = sha256(key);
        k_pad[..32].copy_from_slice(&kh);
    } else {
        k_pad[..key.len()].copy_from_slice(key);
    }
    let mut inner_buf = Vec::with_capacity(64 + message.len());
    for b in k_pad.iter() {
        inner_buf.push(b ^ 0x36);
    }
    inner_buf.extend_from_slice(message);
    let inner_hash = sha256(&inner_buf);

    let mut outer_buf = Vec::with_capacity(64 + 32);
    for b in k_pad.iter() {
        outer_buf.push(b ^ 0x5c);
    }
    outer_buf.extend_from_slice(&inner_hash);
    sha256(&outer_buf)
}

/// Magic bytes for signed Cursor-on-Target wire frame.
pub const COT_SIG_MAGIC: [u8; 4] = *b"CSIG";

/// Version 1 framing header format.
pub const COT_SIG_VERSION: u8 = 1;

/// Fixed length of signed CoT binary header (53 bytes).
pub const COT_HEADER_LEN: usize = 53;

/// Errors encountered during signature generation or verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CotSignerError {
    FrameTooShort { actual: usize, expected: usize },
    InvalidMagic([u8; 4]),
    UnsupportedVersion(u8),
    KeyNotFound(u32),
    InvalidSignature,
    PayloadLengthMismatch { expected: usize, actual: usize },
    TimestampExpired { age_secs: u64, max_skew_secs: u64 },
    TimestampInFuture { diff_secs: u64, max_skew_secs: u64 },
}

impl fmt::Display for CotSignerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FrameTooShort { actual, expected } => {
                write!(
                    f,
                    "frame length {actual} shorter than required header {expected}"
                )
            }
            Self::InvalidMagic(m) => write!(f, "invalid magic {m:?}"),
            Self::UnsupportedVersion(v) => write!(f, "unsupported version {v}"),
            Self::KeyNotFound(k) => write!(f, "signing key {k} not found in store"),
            Self::InvalidSignature => write!(f, "cryptographic signature mismatch"),
            Self::PayloadLengthMismatch { expected, actual } => {
                write!(
                    f,
                    "payload len mismatch: header claims {expected}, got {actual}"
                )
            }
            Self::TimestampExpired {
                age_secs,
                max_skew_secs,
            } => {
                write!(
                    f,
                    "message expired: age {age_secs}s exceeds max skew {max_skew_secs}s"
                )
            }
            Self::TimestampInFuture {
                diff_secs,
                max_skew_secs,
            } => {
                write!(
                    f,
                    "message timestamp in future by {diff_secs}s exceeds {max_skew_secs}s"
                )
            }
        }
    }
}

impl std::error::Error for CotSignerError {}

/// Verified or packaged signed CoT wire frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedCotFrame {
    pub key_id: u32,
    pub timestamp: u64,
    pub signature: [u8; 32],
    pub payload: Vec<u8>,
}

/// Configuration for CoT message signing.
#[derive(Debug, Clone)]
pub struct CotSignerConfig {
    pub key_id: u32,
    pub secret_key: Vec<u8>,
    pub max_clock_skew_secs: u64,
}

impl Default for CotSignerConfig {
    fn default() -> Self {
        Self {
            key_id: 1,
            secret_key: Vec::new(),
            max_clock_skew_secs: 300,
        }
    }
}

/// Statistics counters for signer instance.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CotSignerStats {
    pub signatures_generated: u64,
    pub frames_packaged: u64,
}

/// Statistics counters for signature validator instance.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CotValidatorStats {
    pub frames_processed: u64,
    pub valid_signatures: u64,
    pub invalid_signatures: u64,
    pub key_not_found: u64,
    pub timestamp_expired: u64,
    pub timestamp_future: u64,
    pub malformed_frames: u64,
}

/// Cursor-on-Target cryptographic signature generator.
pub struct CotSigner {
    config: CotSignerConfig,
    stats: CotSignerStats,
}

impl CotSigner {
    pub fn new(config: CotSignerConfig) -> Self {
        Self {
            config,
            stats: CotSignerStats::default(),
        }
    }

    pub fn key_id(&self) -> u32 {
        self.config.key_id
    }

    pub fn stats(&self) -> &CotSignerStats {
        &self.stats
    }
    pub fn sign_raw(&mut self, payload: &[u8]) -> [u8; 32] {
        self.stats.signatures_generated += 1;
        hmac_sha256(&self.config.secret_key, payload)
    }

    pub fn sign_raw_hex(&mut self, payload: &[u8]) -> String {
        let sig = self.sign_raw(payload);
        bytes_to_hex(&sig)
    }
    pub fn create_signed_frame(&mut self, payload: &[u8], timestamp: u64) -> Vec<u8> {
        let mut to_sign = Vec::with_capacity(12 + payload.len());
        to_sign.extend_from_slice(&self.config.key_id.to_be_bytes());
        to_sign.extend_from_slice(&timestamp.to_be_bytes());
        to_sign.extend_from_slice(payload);

        let signature = hmac_sha256(&self.config.secret_key, &to_sign);
        self.stats.signatures_generated += 1;
        self.stats.frames_packaged += 1;

        let mut frame = Vec::with_capacity(COT_HEADER_LEN + payload.len());
        frame.extend_from_slice(&COT_SIG_MAGIC);
        frame.push(COT_SIG_VERSION);
        frame.extend_from_slice(&self.config.key_id.to_be_bytes());
        frame.extend_from_slice(&timestamp.to_be_bytes());
        frame.extend_from_slice(&signature);
        frame.extend_from_slice(&(payload.len() as u32).to_be_bytes());
        frame.extend_from_slice(payload);
        frame
    }
}

/// Cursor-on-Target cryptographic signature validator.
pub struct CotSignatureValidator {
    keys: HashMap<u32, Vec<u8>>,
    max_clock_skew_secs: u64,
    stats: CotValidatorStats,
}

impl CotSignatureValidator {
    pub fn new(max_clock_skew_secs: u64) -> Self {
        Self {
            keys: HashMap::new(),
            max_clock_skew_secs,
            stats: CotValidatorStats::default(),
        }
    }

    pub fn add_key(&mut self, key_id: u32, key: Vec<u8>) {
        self.keys.insert(key_id, key);
    }

    pub fn remove_key(&mut self, key_id: u32) -> Option<Vec<u8>> {
        self.keys.remove(&key_id)
    }

    pub fn stats(&self) -> &CotValidatorStats {
        &self.stats
    }
    pub fn verify_raw(
        &mut self,
        key_id: u32,
        payload: &[u8],
        expected_sig: &[u8; 32],
    ) -> Result<(), CotSignerError> {
        let key = match self.keys.get(&key_id) {
            Some(k) => k,
            None => {
                self.stats.key_not_found += 1;
                return Err(CotSignerError::KeyNotFound(key_id));
            }
        };

        let computed = hmac_sha256(key, payload);
        if constant_time_compare(&computed, expected_sig) {
            self.stats.valid_signatures += 1;
            Ok(())
        } else {
            self.stats.invalid_signatures += 1;
            Err(CotSignerError::InvalidSignature)
        }
    }
    pub fn verify_frame(
        &mut self,
        frame_bytes: &[u8],
        current_time_secs: u64,
    ) -> Result<SignedCotFrame, CotSignerError> {
        self.stats.frames_processed += 1;

        if frame_bytes.len() < COT_HEADER_LEN {
            self.stats.malformed_frames += 1;
            return Err(CotSignerError::FrameTooShort {
                actual: frame_bytes.len(),
                expected: COT_HEADER_LEN,
            });
        }

        let mut magic = [0u8; 4];
        magic.copy_from_slice(&frame_bytes[0..4]);
        if magic != COT_SIG_MAGIC {
            self.stats.malformed_frames += 1;
            return Err(CotSignerError::InvalidMagic(magic));
        }

        let version = frame_bytes[4];
        if version != COT_SIG_VERSION {
            self.stats.malformed_frames += 1;
            return Err(CotSignerError::UnsupportedVersion(version));
        }
        let key_id = u32::from_be_bytes(frame_bytes[5..9].try_into().unwrap());
        let timestamp = u64::from_be_bytes(frame_bytes[9..17].try_into().unwrap());

        let mut signature = [0u8; 32];
        signature.copy_from_slice(&frame_bytes[17..49]);

        let payload_len = u32::from_be_bytes(frame_bytes[49..53].try_into().unwrap()) as usize;
        let payload = &frame_bytes[53..];

        if payload.len() != payload_len {
            self.stats.malformed_frames += 1;
            return Err(CotSignerError::PayloadLengthMismatch {
                expected: payload_len,
                actual: payload.len(),
            });
        }
        if current_time_secs >= timestamp {
            let age = current_time_secs - timestamp;
            if age > self.max_clock_skew_secs {
                self.stats.timestamp_expired += 1;
                return Err(CotSignerError::TimestampExpired {
                    age_secs: age,
                    max_skew_secs: self.max_clock_skew_secs,
                });
            }
        } else {
            let future_diff = timestamp - current_time_secs;
            if future_diff > self.max_clock_skew_secs {
                self.stats.timestamp_future += 1;
                return Err(CotSignerError::TimestampInFuture {
                    diff_secs: future_diff,
                    max_skew_secs: self.max_clock_skew_secs,
                });
            }
        }
        let key = match self.keys.get(&key_id) {
            Some(k) => k,
            None => {
                self.stats.key_not_found += 1;
                return Err(CotSignerError::KeyNotFound(key_id));
            }
        };

        let mut to_verify = Vec::with_capacity(12 + payload.len());
        to_verify.extend_from_slice(&key_id.to_be_bytes());
        to_verify.extend_from_slice(&timestamp.to_be_bytes());
        to_verify.extend_from_slice(payload);

        let computed = hmac_sha256(key, &to_verify);
        if !constant_time_compare(&computed, &signature) {
            self.stats.invalid_signatures += 1;
            return Err(CotSignerError::InvalidSignature);
        }

        self.stats.valid_signatures += 1;
        Ok(SignedCotFrame {
            key_id,
            timestamp,
            signature,
            payload: payload.to_vec(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_nist_standard_vectors() {
        let empty = sha256(b"");
        assert_eq!(
            bytes_to_hex(&empty),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );

        let abc = sha256(b"abc");
        assert_eq!(
            bytes_to_hex(&abc),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }


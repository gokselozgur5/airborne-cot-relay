//! CoT XML payload compressor using Snappy and Deflate stream encoder.

/// Supported compression algorithms for Cursor-on-Target payloads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompressionAlgorithm {
    /// Passthrough uncompressed payload data.
    #[default]
    None = 0,
    /// Snappy raw block/stream format.
    Snappy = 1,
    /// RFC 1951 Deflate stream format.
    Deflate = 2,
}

impl CompressionAlgorithm {
    /// Converts algorithm enum to protocol wire identifier byte.
    pub const fn as_u8(&self) -> u8 {
        *self as u8
    }

    /// Parses protocol wire identifier byte into algorithm enum.
    pub const fn from_u8(val: u8) -> Result<Self, CompressorError> {
        match val {
            0 => Ok(Self::None),
            1 => Ok(Self::Snappy),
            2 => Ok(Self::Deflate),
            _ => Err(CompressorError::InvalidAlgorithm(val)),
        }
    }
}

impl CompressionAlgorithm {
    /// Returns human-readable lowercase algorithm name.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Snappy => "snappy",
            Self::Deflate => "deflate",
        }
    }
}

/// Error conditions encountered during payload compression or decompression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompressorError {
    /// Unrecognized algorithm wire identifier.
    InvalidAlgorithm(u8),
    /// Payload exceeds configured maximum byte limit.
    PayloadTooLarge { len: usize, max: usize },
    /// Corrupt compressed payload or invalid control code.
    CorruptPayload(&'static str),
    /// Decompression size mismatch or stream termination error.
    DecompressionFailed(&'static str),
    /// Frame magic header mismatch.
    HeaderMismatch,
    /// Decompressed payload is not valid UTF-8 string.
    InvalidUtf8,
}

impl std::fmt::Display for CompressorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidAlgorithm(code) => write!(f, "invalid algorithm tag: {code}"),
            Self::PayloadTooLarge { len, max } => {
                write!(f, "payload length {len} exceeds max {max}")
            }
            Self::CorruptPayload(msg) => write!(f, "corrupt payload: {msg}"),
            Self::DecompressionFailed(msg) => write!(f, "decompression failed: {msg}"),
            Self::HeaderMismatch => write!(f, "invalid frame magic header"),
            Self::InvalidUtf8 => write!(f, "decompressed payload is not valid utf-8"),
        }
    }
}

impl std::error::Error for CompressorError {}

/// Configuration settings for CoT payload compressor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompressorConfig {
    /// Selected compression algorithm.
    pub algorithm: CompressionAlgorithm,
    /// Minimum size in bytes before compression is applied.
    pub min_compress_bytes: usize,
    /// Maximum allowed payload size in bytes.
    pub max_payload_bytes: usize,
}

impl Default for CompressorConfig {
    fn default() -> Self {
        Self {
            algorithm: CompressionAlgorithm::Snappy,
            min_compress_bytes: 32,
            max_payload_bytes: 10 * 1024 * 1024,
        }
    }
}

impl CompressorConfig {
    /// Sets compression algorithm.
    pub fn with_algorithm(mut self, algorithm: CompressionAlgorithm) -> Self {
        self.algorithm = algorithm;
        self
    }

    /// Sets minimum compression threshold.
    pub fn with_min_compress_bytes(mut self, threshold: usize) -> Self {
        self.min_compress_bytes = threshold;
        self
    }

    /// Sets maximum payload limit.
    pub fn with_max_payload_bytes(mut self, max_bytes: usize) -> Self {
        self.max_payload_bytes = max_bytes;
        self
    }
}

/// Compressed payload container with metadata and framed wire serialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompressedPayload {
    /// Algorithm used to encode payload bytes.
    pub algorithm: CompressionAlgorithm,
    /// Original uncompressed size in bytes.
    pub uncompressed_len: usize,
    /// Encoded payload data bytes.
    pub data: Vec<u8>,
}

impl CompressedPayload {
    /// Magic header bytes identifying CoT compressed payloads.
    pub const MAGIC: [u8; 2] = [0x43, 0x50];

    /// Calculates compression ratio (compressed / uncompressed).
    pub fn ratio(&self) -> f64 {
        if self.uncompressed_len == 0 {
            return 1.0;
        }
        self.data.len() as f64 / self.uncompressed_len as f64
    }

    /// Returns true if algorithm is not None.
    pub const fn is_compressed(&self) -> bool {
        !matches!(self.algorithm, CompressionAlgorithm::None)
    }
}

impl CompressedPayload {
    /// Serializes payload into framed wire format with header and length prefix.
    pub fn to_framed_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(7 + self.data.len());
        out.extend_from_slice(&Self::MAGIC);
        out.push(self.algorithm.as_u8());
        out.extend_from_slice(&(self.uncompressed_len as u32).to_be_bytes());
        out.extend_from_slice(&self.data);
        out
    }
}

impl CompressedPayload {
    /// Deserializes payload from framed wire bytes.
    pub fn from_framed_bytes(buf: &[u8]) -> Result<Self, CompressorError> {
        if buf.len() < 7 {
            return Err(CompressorError::CorruptPayload("frame too short"));
        }
        if buf[0] != Self::MAGIC[0] || buf[1] != Self::MAGIC[1] {
            return Err(CompressorError::HeaderMismatch);
        }
        let algorithm = CompressionAlgorithm::from_u8(buf[2])?;
        let uncompressed_len = u32::from_be_bytes([buf[3], buf[4], buf[5], buf[6]]) as usize;
        let data = buf[7..].to_vec();
        Ok(Self { algorithm, uncompressed_len, data })
    }
}

/// Statistics tracking cumulative compression throughput and bandwidth savings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CompressorStats {
    /// Cumulative uncompressed input bytes processed.
    pub total_raw_bytes: u64,
    /// Cumulative compressed output bytes emitted.
    pub total_compressed_bytes: u64,
    /// Total count of payloads compressed.
    pub payloads_compressed: u64,
    /// Total count of payloads bypassed due to threshold.
    pub payloads_bypassed: u64,
}

impl CompressorStats {
    /// Returns overall compression ratio.
    pub fn overall_ratio(&self) -> f64 {
        if self.total_raw_bytes == 0 {
            return 1.0;
        }
        self.total_compressed_bytes as f64 / self.total_raw_bytes as f64
    }

    /// Returns bandwidth percentage saved.
    pub fn savings_percent(&self) -> f64 {
        (1.0 - self.overall_ratio()) * 100.0
    }
}

fn encode_varint_u32(mut val: u32, out: &mut Vec<u8>) {
    while val >= 0x80 {
        out.push(((val & 0x7f) | 0x80) as u8);
        val >>= 7;
    }
    out.push(val as u8);
}

fn decode_varint_u32(input: &[u8]) -> Result<(u32, usize), CompressorError> {
    let mut val: u32 = 0;
    let mut shift = 0;
    for (i, &b) in input.iter().enumerate() {
        if shift >= 32 {
            return Err(CompressorError::CorruptPayload("varint overflow"));
        }
        val |= ((b & 0x7f) as u32) << shift;
        if (b & 0x80) == 0 {
            return Ok((val, i + 1));
        }
        shift += 7;
    }
    Err(CompressorError::CorruptPayload("truncated varint"))
}

fn emit_snappy_literal(lit: &[u8], out: &mut Vec<u8>) {
    if lit.is_empty() {
        return;
    }
    let n = lit.len();
    if n <= 60 {
        out.push(((n - 1) as u8) << 2);
    } else if n <= 256 {
        out.push(60 << 2);
        out.push((n - 1) as u8);
    } else if n <= 65536 {
        out.push(61 << 2);
        out.extend_from_slice(&((n - 1) as u16).to_le_bytes());
    } else {
        out.push(62 << 2);
        let bytes = ((n - 1) as u32).to_le_bytes();
        out.extend_from_slice(&bytes[..3]);
    }
    out.extend_from_slice(lit);
}

fn emit_snappy_copy(offset: usize, len: usize, out: &mut Vec<u8>) {
    if (4..=11).contains(&len) && offset < 2048 {
        let tag = 0x01 | (((len - 4) as u8) << 2) | (((offset >> 8) as u8 & 0x07) << 5);
        out.push(tag);
        out.push((offset & 0xff) as u8);
    } else if offset <= 65535 {
        let tag = 0x02 | (((len - 1) as u8) << 2);
        out.push(tag);
        out.extend_from_slice(&(offset as u16).to_le_bytes());
    } else {
        let tag = 0x03 | (((len - 1) as u8) << 2);
        out.push(tag);
        out.extend_from_slice(&(offset as u32).to_le_bytes());
    }
}

fn find_snappy_match(input: &[u8], pos: usize, cand: usize) -> usize {
    let mut match_len = 4;
    while pos + match_len < input.len() && match_len < 64 && input[cand + match_len] == input[pos + match_len] {
        match_len += 1;
    }
    match_len
}

fn step_snappy_compress(
    input: &[u8],
    pos: &mut usize,
    lit_start: &mut usize,
    table: &mut [u32; 8192],
    out: &mut Vec<u8>,
) {
    let p = *pos;
    let chunk = u32::from_le_bytes([input[p], input[p + 1], input[p + 2], input[p + 3]]);
    let hash = ((chunk.wrapping_mul(0x1e35a7bd)) >> 19) as usize & 8191;
    let cand = table[hash] as usize;
    table[hash] = p as u32;
    if p > cand && (p - cand) <= 65535 && input[cand..cand + 4] == input[p..p + 4] {
        let offset = p - cand;
        let match_len = find_snappy_match(input, p, cand);
        if p > *lit_start {
            emit_snappy_literal(&input[*lit_start..p], out);
        }
        emit_snappy_copy(offset, match_len, out);
        *pos += match_len;
        *lit_start = *pos;
    } else {
        *pos += 1;
    }
}

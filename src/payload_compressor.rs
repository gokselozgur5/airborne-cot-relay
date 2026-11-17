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

/// Compresses payload into Snappy raw block format.
pub fn snappy_compress(input: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len() / 2 + 16);
    encode_varint_u32(input.len() as u32, &mut out);
    if input.is_empty() {
        return out;
    }
    let mut table = [0u32; 8192];
    let mut pos = 0;
    let mut lit_start = 0;
    while pos + 4 <= input.len() {
        step_snappy_compress(input, &mut pos, &mut lit_start, &mut table, &mut out);
    }
    if lit_start < input.len() {
        emit_snappy_literal(&input[lit_start..], &mut out);
    }
    out
}

fn decode_snappy_literal_len(tag: u8, input: &[u8], cursor: &mut usize) -> Result<usize, CompressorError> {
    let high = tag >> 2;
    match high {
        0..=59 => Ok((high as usize) + 1),
        60 => {
            if *cursor >= input.len() {
                return Err(CompressorError::CorruptPayload("truncated literal length"));
            }
            let val = input[*cursor] as usize + 1;
            *cursor += 1;
            Ok(val)
        }
        61 => {
            if *cursor + 2 > input.len() {
                return Err(CompressorError::CorruptPayload("truncated literal length"));
            }
            let val = u16::from_le_bytes([input[*cursor], input[*cursor + 1]]) as usize + 1;
            *cursor += 2;
            Ok(val)
        }
        _ => Err(CompressorError::CorruptPayload("unsupported literal length code")),
    }
}

fn copy_snappy_bytes(out: &mut Vec<u8>, offset: usize, len: usize) -> Result<(), CompressorError> {
    if offset == 0 || offset > out.len() {
        return Err(CompressorError::CorruptPayload("invalid copy offset"));
    }
    let start = out.len() - offset;
    for i in 0..len {
        let b = out[start + i];
        out.push(b);
    }
    Ok(())
}

fn decode_snappy_copy_tag(tag: u8, input: &[u8], cursor: &mut usize, out: &mut Vec<u8>) -> Result<(), CompressorError> {
    match tag & 0x03 {
        0x01 => {
            if *cursor >= input.len() {
                return Err(CompressorError::CorruptPayload("truncated copy offset"));
            }
            let len = (((tag >> 2) & 0x07) as usize) + 4;
            let offset = ((((tag >> 5) & 0x07) as usize) << 8) | (input[*cursor] as usize);
            *cursor += 1;
            copy_snappy_bytes(out, offset, len)
        }
        0x02 => {
            if *cursor + 2 > input.len() {
                return Err(CompressorError::CorruptPayload("truncated 2-byte copy offset"));
            }
            let len = ((tag >> 2) as usize) + 1;
            let offset = u16::from_le_bytes([input[*cursor], input[*cursor + 1]]) as usize;
            *cursor += 2;
            copy_snappy_bytes(out, offset, len)
        }
        _ => Err(CompressorError::CorruptPayload("unsupported copy tag")),
    }
}

fn decode_snappy_step(tag: u8, input: &[u8], cursor: &mut usize, out: &mut Vec<u8>) -> Result<(), CompressorError> {
    if (tag & 0x03) == 0x00 {
        let lit_len = decode_snappy_literal_len(tag, input, cursor)?;
        if *cursor + lit_len > input.len() {
            return Err(CompressorError::CorruptPayload("literal overrun"));
        }
        out.extend_from_slice(&input[*cursor..*cursor + lit_len]);
        *cursor += lit_len;
        Ok(())
    } else {
        decode_snappy_copy_tag(tag, input, cursor, out)
    }
}

/// Decompresses Snappy raw block payload into uncompressed bytes.
pub fn snappy_decompress(input: &[u8]) -> Result<Vec<u8>, CompressorError> {
    let (expected_len, mut cursor) = decode_varint_u32(input)?;
    let mut out = Vec::with_capacity(expected_len as usize);
    while cursor < input.len() && out.len() < expected_len as usize {
        let tag = input[cursor];
        cursor += 1;
        decode_snappy_step(tag, input, &mut cursor, &mut out)?;
    }
    if out.len() != expected_len as usize {
        return Err(CompressorError::DecompressionFailed("snappy length mismatch"));
    }
    Ok(out)
}

struct BitWriter {
    buf: Vec<u8>,
    bit_buf: u32,
    bits: u8,
}

impl BitWriter {
    fn new() -> Self {
        Self { buf: Vec::new(), bit_buf: 0, bits: 0 }
    }
}

impl BitWriter {
    fn write_bits(&mut self, val: u32, count: u8) {
        self.bit_buf |= (val & ((1 << count) - 1)) << self.bits;
        self.bits += count;
        while self.bits >= 8 {
            self.buf.push(self.bit_buf as u8);
            self.bit_buf >>= 8;
            self.bits -= 8;
        }
    }
    fn flush_byte(&mut self) {
        if self.bits > 0 {
            self.buf.push(self.bit_buf as u8);
            self.bit_buf = 0;
            self.bits = 0;
        }
    }
}

struct BitReader<'a> {
    data: &'a [u8],
    pos: usize,
    bit_buf: u32,
    bits: u8,
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0, bit_buf: 0, bits: 0 }
    }
}

impl<'a> BitReader<'a> {
    fn read_bits(&mut self, count: u8) -> Result<u32, CompressorError> {
        while self.bits < count {
            if self.pos >= self.data.len() {
                return Err(CompressorError::CorruptPayload("unexpected eof in bitstream"));
            }
            self.bit_buf |= (self.data[self.pos] as u32) << self.bits;
            self.pos += 1;
            self.bits += 8;
        }
        let val = self.bit_buf & ((1 << count) - 1);
        self.bit_buf >>= count;
        self.bits -= count;
        Ok(val)
    }
}

fn rev_bits(mut code: u32, len: u8) -> u32 {
    let mut res = 0;
    for _ in 0..len {
        res = (res << 1) | (code & 1);
        code >>= 1;
    }
    res
}

fn fixed_lit_code(sym: u16) -> (u32, u8) {
    if sym <= 143 {
        (rev_bits(0x30 + sym as u32, 8), 8)
    } else if sym <= 255 {
        (rev_bits(0x190 + (sym - 144) as u32, 9), 9)
    } else if sym <= 279 {
        (rev_bits((sym - 256) as u32, 7), 7)
    } else {
        (rev_bits(0xc0 + (sym - 280) as u32, 8), 8)
    }
}

const LENGTH_BASE: [u16; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131, 163, 195, 227, 258,
];
const LENGTH_EXTRA: [u8; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];

fn length_to_code(len: usize) -> (u16, u32, u8) {
    let len = len.min(258) as u16;
    for i in (0..29).rev() {
        if len >= LENGTH_BASE[i] {
            let extra_bits = LENGTH_EXTRA[i];
            let extra_val = (len - LENGTH_BASE[i]) as u32;
            return (257 + i as u16, extra_val, extra_bits);
        }
    }
    (257, 0, 0)
}

const DIST_BASE: [u16; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537, 2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];
const DIST_EXTRA: [u8; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13,
];

fn dist_to_code(dist: usize) -> (u8, u32, u8) {
    let dist = dist.min(32768) as u16;
    for i in (0..30).rev() {
        if dist >= DIST_BASE[i] {
            let extra_bits = DIST_EXTRA[i];
            let extra_val = (dist - DIST_BASE[i]) as u32;
            return (i as u8, extra_val, extra_bits);
        }
    }
    (0, 0, 0)
}

fn write_deflate_match(writer: &mut BitWriter, match_len: usize, offset: usize) {
    let (len_code, len_extra_val, len_extra_bits) = length_to_code(match_len);
    let (code_bits, code_len) = fixed_lit_code(len_code);
    writer.write_bits(code_bits, code_len);
    if len_extra_bits > 0 {
        writer.write_bits(len_extra_val, len_extra_bits);
    }
    let (dist_code, dist_extra_val, dist_extra_bits) = dist_to_code(offset);
    let dist_rev = rev_bits(dist_code as u32, 5);
    writer.write_bits(dist_rev, 5);
    if dist_extra_bits > 0 {
        writer.write_bits(dist_extra_val, dist_extra_bits);
    }
}

fn step_deflate_compress(input: &[u8], pos: &mut usize, table: &mut [u32; 4096], writer: &mut BitWriter) {
    let p = *pos;
    if p + 3 <= input.len() {
        let chunk = (input[p] as u32) | ((input[p + 1] as u32) << 8) | ((input[p + 2] as u32) << 16);
        let hash = ((chunk.wrapping_mul(0x1e35a7bd)) >> 20) as usize & 4095;
        let cand = table[hash] as usize;
        table[hash] = p as u32;
        if p > cand && (p - cand) <= 32768 && input[cand..cand + 3] == input[p..p + 3] {
            let offset = p - cand;
            let mut match_len = 3;
            while p + match_len < input.len() && match_len < 258 && input[cand + match_len] == input[p + match_len] {
                match_len += 1;
            }
            write_deflate_match(writer, match_len, offset);
            *pos += match_len;
            return;
        }
    }
    let (c, l) = fixed_lit_code(input[p] as u16);
    writer.write_bits(c, l);
    *pos += 1;
}

/// Compresses payload into RFC 1951 Deflate stream format using Fixed Huffman.
pub fn deflate_compress(input: &[u8]) -> Vec<u8> {
    let mut writer = BitWriter::new();
    writer.write_bits(1, 1);
    writer.write_bits(1, 2);
    let mut pos = 0;
    let mut table = [0u32; 4096];
    while pos < input.len() {
        step_deflate_compress(input, &mut pos, &mut table, &mut writer);
    }
    let (c, l) = fixed_lit_code(256);
    writer.write_bits(c, l);
    writer.flush_byte();
    writer.buf
}

fn decode_fixed_huffman_sym(reader: &mut BitReader<'_>) -> Result<u16, CompressorError> {
    let b7 = reader.read_bits(7)?;
    let b7_rev = rev_bits(b7, 7);
    if b7_rev <= 23 {
        Ok(256 + b7_rev as u16)
    } else {
        let b8_bit = reader.read_bits(1)?;
        let b8 = (b8_bit << 7) | b7;
        let b8_rev = rev_bits(b8, 8);
        if (0x30..=0xbf).contains(&b8_rev) {
            Ok((b8_rev - 0x30) as u16)
        } else if (0xc0..=0xc7).contains(&b8_rev) {
            Ok((280 + (b8_rev - 0xc0)) as u16)
        } else {
            let b9_bit = reader.read_bits(1)?;
            let b9 = (b9_bit << 8) | b8;
            let b9_rev = rev_bits(b9, 9);
            Ok((144 + (b9_rev - 0x190)) as u16)
        }
    }
}

fn decode_deflate_copy(reader: &mut BitReader<'_>, out: &mut Vec<u8>, sym: u16) -> Result<(), CompressorError> {
    let len_idx = (sym - 257) as usize;
    if len_idx >= 29 { return Err(CompressorError::CorruptPayload("invalid length code")); }
    let extra_bits = LENGTH_EXTRA[len_idx];
    let extra_val = if extra_bits > 0 { reader.read_bits(extra_bits)? } else { 0 };
    let length = LENGTH_BASE[len_idx] as usize + extra_val as usize;
    let dist_bits = reader.read_bits(5)?;
    let dist_code = rev_bits(dist_bits, 5) as usize;
    if dist_code >= 30 { return Err(CompressorError::CorruptPayload("invalid distance code")); }
    let dist_extra_bits = DIST_EXTRA[dist_code];
    let dist_extra_val = if dist_extra_bits > 0 { reader.read_bits(dist_extra_bits)? } else { 0 };
    let distance = DIST_BASE[dist_code] as usize + dist_extra_val as usize;
    if distance == 0 || distance > out.len() {
        return Err(CompressorError::CorruptPayload("invalid backward distance"));
    }
    let start = out.len() - distance;
    for i in 0..length {
        let b = out[start + i];
        out.push(b);
    }
    Ok(())
}

/// Decompresses RFC 1951 Deflate stream payload into uncompressed bytes.
pub fn deflate_decompress(input: &[u8]) -> Result<Vec<u8>, CompressorError> {
    let mut reader = BitReader::new(input);
    let _bfinal = reader.read_bits(1)?;
    let btype = reader.read_bits(2)?;
    if btype != 1 {
        return Err(CompressorError::CorruptPayload("unsupported deflate btype"));
    }
    let mut out = Vec::new();
    loop {
        let sym = decode_fixed_huffman_sym(&mut reader)?;
        if sym == 256 {
            break;
        } else if sym < 256 {
            out.push(sym as u8);
        } else {
            decode_deflate_copy(&mut reader, &mut out, sym)?;
        }
    }
    Ok(out)
}

/// Primary CoT payload compression engine with configurable algorithms and stats.
#[derive(Debug, Clone)]
pub struct CotPayloadCompressor {
    config: CompressorConfig,
    stats: CompressorStats,
}

impl CotPayloadCompressor {
    /// Creates a new compressor with the given configuration.
    pub fn new(config: CompressorConfig) -> Self {
        Self {
            config,
            stats: CompressorStats::default(),
        }
    }
}

impl CotPayloadCompressor {
    /// Returns current configuration.
    pub fn config(&self) -> &CompressorConfig {
        &self.config
    }

    /// Returns cumulative compression metrics.
    pub fn stats(&self) -> &CompressorStats {
        &self.stats
    }

    /// Resets cumulative stats counters.
    pub fn reset_stats(&mut self) {
        self.stats = CompressorStats::default();
    }
}

impl CotPayloadCompressor {
    fn should_bypass(&self, len: usize) -> bool {
        len < self.config.min_compress_bytes || self.config.algorithm == CompressionAlgorithm::None
    }

    fn execute_compression(&self, payload: &[u8]) -> Vec<u8> {
        match self.config.algorithm {
            CompressionAlgorithm::Snappy => snappy_compress(payload),
            CompressionAlgorithm::Deflate => deflate_compress(payload),
            CompressionAlgorithm::None => payload.to_vec(),
        }
    }
}

impl CotPayloadCompressor {
    fn record_bypassed_payload(&mut self, len: usize, payload: &[u8]) -> CompressedPayload {
        self.stats.payloads_bypassed += 1;
        self.stats.total_compressed_bytes += len as u64;
        CompressedPayload {
            algorithm: CompressionAlgorithm::None,
            uncompressed_len: len,
            data: payload.to_vec(),
        }
    }
}

impl CotPayloadCompressor {
    /// Compresses a raw byte payload according to configured algorithm and threshold.
    pub fn compress(&mut self, payload: &[u8]) -> Result<CompressedPayload, CompressorError> {
        let len = payload.len();
        if len > self.config.max_payload_bytes {
            return Err(CompressorError::PayloadTooLarge {
                len,
                max: self.config.max_payload_bytes,
            });
        }
        self.stats.total_raw_bytes += len as u64;
        if self.should_bypass(len) {
            return Ok(self.record_bypassed_payload(len, payload));
        }
        let data = self.execute_compression(payload);
        self.stats.payloads_compressed += 1;
        self.stats.total_compressed_bytes += data.len() as u64;
        Ok(CompressedPayload {
            algorithm: self.config.algorithm,
            uncompressed_len: len,
            data,
        })
    }
}

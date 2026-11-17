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

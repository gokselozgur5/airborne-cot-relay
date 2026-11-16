//! Length-prefixed stream frame delimiter codec.

/// Error conditions encountered during frame decoding or encoding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameCodecError {
    /// Frame payload length exceeds configured maximum limit.
    FrameTooLarge { frame_len: usize, max_len: usize },
    /// Length prefix field could not be parsed or is invalid.
    InvalidPrefix(&'static str),
}

impl std::fmt::Display for FrameCodecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FrameTooLarge { frame_len, max_len } => {
                write!(f, "frame length {frame_len} exceeds max {max_len}")
            }
            Self::InvalidPrefix(msg) => write!(f, "invalid length prefix: {msg}"),
        }
    }
}

impl std::error::Error for FrameCodecError {}

/// Width of the frame length prefix field in bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LengthField {
    /// 2-byte unsigned integer (max 65535 bytes).
    U16,
    /// 4-byte unsigned integer (max ~4GB bytes).
    #[default]
    U32,
}

impl LengthField {
    /// Number of bytes occupied by this length prefix.
    pub const fn byte_len(&self) -> usize {
        match self {
            Self::U16 => 2,
            Self::U32 => 4,
        }
    }
}

/// Byte order for the length prefix field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Endianness {
    /// Big-endian network byte order (default).
    #[default]
    BigEndian,
    /// Little-endian host byte order.
    LittleEndian,
}

/// Streaming length-delimited frame decoder.
#[derive(Debug, Clone)]
pub struct FrameDecoder {
    max_frame_len: usize,
    length_field: LengthField,
    endianness: Endianness,
    buffer: Vec<u8>,
}

impl FrameDecoder {
    /// Constructs a new `FrameDecoder` with default 4-byte big-endian prefix.
    pub fn new(max_frame_len: usize) -> Self {
        Self {
            max_frame_len,
            length_field: LengthField::default(),
            endianness: Endianness::default(),
            buffer: Vec::new(),
        }
    }
}

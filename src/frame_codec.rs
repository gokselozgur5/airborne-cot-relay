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

impl FrameDecoder {
    /// Configures the length prefix field width.
    pub fn with_length_field(mut self, length_field: LengthField) -> Self {
        self.length_field = length_field;
        self
    }

    /// Configures the byte order for length parsing.
    pub fn with_endianness(mut self, endianness: Endianness) -> Self {
        self.endianness = endianness;
        self
    }
}

impl FrameDecoder {
    /// Returns the maximum allowed frame length.
    pub const fn max_frame_len(&self) -> usize {
        self.max_frame_len
    }

    /// Returns configured length prefix format.
    pub const fn length_field(&self) -> LengthField {
        self.length_field
    }

    /// Returns configured endianness.
    pub const fn endianness(&self) -> Endianness {
        self.endianness
    }

    /// Number of buffered unparsed bytes.
    pub fn buffer_len(&self) -> usize {
        self.buffer.len()
    }

    /// Clears any unparsed buffered bytes.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

impl FrameDecoder {
    /// Appends incoming streaming bytes to the internal decode buffer.
    pub fn feed(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
    }

    fn parse_length(bytes: &[u8], field: LengthField, endian: Endianness) -> usize {
        match (field, endian) {
            (LengthField::U16, Endianness::BigEndian) => {
                u16::from_be_bytes([bytes[0], bytes[1]]) as usize
            }
            (LengthField::U16, Endianness::LittleEndian) => {
                u16::from_le_bytes([bytes[0], bytes[1]]) as usize
            }
            (LengthField::U32, Endianness::BigEndian) => {
                u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize
            }
            (LengthField::U32, Endianness::LittleEndian) => {
                u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize
            }
        }
    }
}

impl FrameDecoder {
    /// Attempts to decode the next frame from the internal buffer.
    pub fn decode_next(&mut self) -> Result<Option<Vec<u8>>, FrameCodecError> {
        let prefix_len = self.length_field.byte_len();
        if self.buffer.len() < prefix_len {
            return Ok(None);
        }
        let frame_len = Self::parse_length(
            &self.buffer[..prefix_len],
            self.length_field,
            self.endianness,
        );
        if frame_len > self.max_frame_len {
            return Err(FrameCodecError::FrameTooLarge {
                frame_len,
                max_len: self.max_frame_len,
            });
        }
        let total_len = prefix_len + frame_len;
        if self.buffer.len() < total_len {
            return Ok(None);
        }
        let payload = self.buffer[prefix_len..total_len].to_vec();
        self.buffer.drain(..total_len);
        Ok(Some(payload))
    }
}

impl FrameDecoder {
    /// Feeds a chunk and drains all fully available frames.
    pub fn decode_all(&mut self, data: &[u8]) -> Result<Vec<Vec<u8>>, FrameCodecError> {
        self.feed(data);
        let mut frames = Vec::new();
        while let Some(frame) = self.decode_next()? {
            frames.push(frame);
        }
        Ok(frames)
    }
}

impl FrameDecoder {
    /// Encodes a payload with the specified prefix configuration.
    pub fn encode(
        payload: &[u8],
        field: LengthField,
        endian: Endianness,
    ) -> Result<Vec<u8>, FrameCodecError> {
        let len = payload.len();
        let mut out = Vec::with_capacity(field.byte_len() + len);
        match (field, endian) {
            (LengthField::U16, Endianness::BigEndian) => {
                let val = u16::try_from(len).map_err(|_| FrameCodecError::FrameTooLarge {
                    frame_len: len,
                    max_len: u16::MAX as usize,
                })?;
                out.extend_from_slice(&val.to_be_bytes());
            }
            (LengthField::U16, Endianness::LittleEndian) => {
                let val = u16::try_from(len).map_err(|_| FrameCodecError::FrameTooLarge {
                    frame_len: len,
                    max_len: u16::MAX as usize,
                })?;
                out.extend_from_slice(&val.to_le_bytes());
            }
            (LengthField::U32, Endianness::BigEndian) => {
                let val = u32::try_from(len).map_err(|_| FrameCodecError::FrameTooLarge {
                    frame_len: len,
                    max_len: u32::MAX as usize,
                })?;
                out.extend_from_slice(&val.to_be_bytes());
            }
            (LengthField::U32, Endianness::LittleEndian) => {
                let val = u32::try_from(len).map_err(|_| FrameCodecError::FrameTooLarge {
                    frame_len: len,
                    max_len: u32::MAX as usize,
                })?;
                out.extend_from_slice(&val.to_le_bytes());
            }
        }
        out.extend_from_slice(payload);
        Ok(out)
    }
}

impl FrameDecoder {
    /// Encodes a payload using this decoder instance settings.
    pub fn encode_frame(&self, payload: &[u8]) -> Result<Vec<u8>, FrameCodecError> {
        if payload.len() > self.max_frame_len {
            return Err(FrameCodecError::FrameTooLarge {
                frame_len: payload.len(),
                max_len: self.max_frame_len,
            });
        }
        Self::encode(payload, self.length_field, self.endianness)
    }
}

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

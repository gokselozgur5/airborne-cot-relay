//! Length-prefixed stream frame delimiter codec.

/// Error conditions encountered during frame decoding or encoding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameCodecError {
    /// Frame payload length exceeds configured maximum limit.
    FrameTooLarge { frame_len: usize, max_len: usize },
    /// Length prefix field could not be parsed or is invalid.
    InvalidPrefix(&'static str),
}

use core::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum RelayError {
    InvalidConfig(String),
    DecodeError(String),
    TargetingError(String),
    IoError(String),
}

impl fmt::Display for RelayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfig(msg) => write!(f, "Invalid config: {msg}"),
            Self::DecodeError(msg) => write!(f, "Decode error: {msg}"),
            Self::TargetingError(msg) => write!(f, "Targeting error: {msg}"),
            Self::IoError(msg) => write!(f, "IO error: {msg}"),
        }
    }
}

impl std::error::Error for RelayError {}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraIntrinsics {
    pub focal_length_px: f64,
    pub principal_u: f64,
    pub principal_v: f64,
    pub image_width: u32,
    pub image_height: u32,
}

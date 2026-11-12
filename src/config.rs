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

impl Default for CameraIntrinsics {
    fn default() -> Self {
        Self {
            focal_length_px: 1200.0,
            principal_u: 640.0,
            principal_v: 360.0,
            image_width: 1280,
            image_height: 720,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RelayConfig {
    pub telemetry_listen_addr: String,
    pub cot_broadcast_addr: String,
    pub camera: CameraIntrinsics,
    pub ground_elevation_m: f64,
    pub target_stale_timeout_ms: u64,
}

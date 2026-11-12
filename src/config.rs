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

impl Default for RelayConfig {
    fn default() -> Self {
        Self {
            telemetry_listen_addr: "0.0.0.0:14550".to_string(),
            cot_broadcast_addr: "239.2.3.1:6969".to_string(),
            camera: CameraIntrinsics::default(),
            ground_elevation_m: 0.0,
            target_stale_timeout_ms: 5000,
        }
    }
}

impl RelayConfig {
    pub fn validate(&self) -> Result<(), RelayError> {
        if self.telemetry_listen_addr.is_empty() {
            return Err(RelayError::InvalidConfig(
                "empty telemetry listen addr".into(),
            ));
        }
        if self.cot_broadcast_addr.is_empty() {
            return Err(RelayError::InvalidConfig("empty cot broadcast addr".into()));
        }
        if self.camera.focal_length_px <= 0.0 {
            return Err(RelayError::InvalidConfig(
                "focal length must be positive".into(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_validity() {
        let config = RelayConfig::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.ground_elevation_m, 0.0);
        assert_eq!(config.target_stale_timeout_ms, 5000);
    }

    #[test]
    fn test_invalid_config_cases() {
        let mut cfg = RelayConfig::default();
        cfg.telemetry_listen_addr.clear();
        assert!(matches!(cfg.validate(), Err(RelayError::InvalidConfig(_))));

        let mut cfg2 = RelayConfig::default();
        cfg2.camera.focal_length_px = 0.0;
        assert!(matches!(cfg2.validate(), Err(RelayError::InvalidConfig(_))));
    }

//! Layered configuration precedence and field-level provenance.

/// Provenance of a configuration field value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Provenance {
    Default,
    File,
    Env,
    Cli,
}

impl std::fmt::Display for Provenance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Default => write!(f, "default"),
            Self::File => write!(f, "file"),
            Self::Env => write!(f, "env"),
            Self::Cli => write!(f, "cli"),
        }
    }
}

/// A value paired with its origin layer.
#[derive(Debug, Clone, PartialEq)]
pub struct LayeredValue<T> {
    pub value: T,
    pub provenance: Provenance,
}

impl<T> LayeredValue<T> {
    pub fn new(value: T, provenance: Provenance) -> Self {
        Self { value, provenance }
    }
}

use crate::config::{CameraIntrinsics, RelayConfig, RelayError};
use crate::config_file::lexer::ParsedIni;
use crate::config_file::map_relay::map_relay_config;

/// Provenance tracking for camera intrinsics fields.
#[derive(Debug, Clone, PartialEq)]
pub struct CameraLayers {
    pub focal_length_px: LayeredValue<f64>,
    pub principal_u: LayeredValue<f64>,
    pub principal_v: LayeredValue<f64>,
    pub image_width: LayeredValue<u32>,
    pub image_height: LayeredValue<u32>,
}

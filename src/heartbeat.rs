//! Airborne telemetry heartbeat generator with platform status and radio metrics.

use crate::PlatformPose;
use sightline_cot::cot::{escape_xml_attr, format_iso8601_utc};

/// Operational state of the airborne platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlatformOperationalState {
    #[default]
    Nominal,
    Degraded,
    Emergency,
    Standby,
    Fault,
}

impl PlatformOperationalState {
    /// Return the string representation of the state.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Nominal => "NOMINAL",
            Self::Degraded => "DEGRADED",
            Self::Emergency => "EMERGENCY",
            Self::Standby => "STANDBY",
            Self::Fault => "FAULT",
        }
    }

    /// Check if the platform is in a healthy operating state.
    pub const fn is_healthy(&self) -> bool {
        matches!(self, Self::Nominal | Self::Standby)
    }
}

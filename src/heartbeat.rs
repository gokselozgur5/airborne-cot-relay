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

/// Airborne platform status and health metrics.
#[derive(Debug, Clone, PartialEq)]
pub struct PlatformMetrics {
    pub callsign: String,
    pub state: PlatformOperationalState,
    pub battery_pct: f32,
    pub battery_voltage_v: f32,
    pub cpu_temp_c: f32,
    pub cpu_load_pct: f32,
    pub uptime_secs: u64,
    pub pose: Option<PlatformPose>,
}

impl PlatformMetrics {
    /// Create a new platform metrics instance.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        callsign: impl Into<String>,
        state: PlatformOperationalState,
        battery_pct: f32,
        battery_voltage_v: f32,
        cpu_temp_c: f32,
        cpu_load_pct: f32,
        uptime_secs: u64,
        pose: Option<PlatformPose>,
    ) -> Self {
        Self {
            callsign: callsign.into(),
            state,
            battery_pct,
            battery_voltage_v,
            cpu_temp_c,
            cpu_load_pct,
            uptime_secs,
            pose,
        }
    }
}

impl PlatformMetrics {
    /// Check if battery is below the given percentage threshold.
    pub fn is_battery_low(&self, threshold_pct: f32) -> bool {
        self.battery_pct < threshold_pct
    }

    /// Check if CPU temperature exceeds the thermal limit.
    pub fn is_cpu_overheating(&self, threshold_c: f32) -> bool {
        self.cpu_temp_c > threshold_c
    }

    /// Update pose coordinates.
    pub fn set_pose(&mut self, pose: Option<PlatformPose>) {
        self.pose = pose;
    }
}

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

/// Radio telemetry link metrics and packet counters.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RadioMetrics {
    pub rssi_dbm: i16,
    pub snr_db: f32,
    pub tx_power_dbm: i16,
    pub tx_packets: u64,
    pub rx_packets: u64,
    pub tx_bytes: u64,
    pub rx_bytes: u64,
    pub tx_errors: u64,
    pub rx_errors: u64,
    pub link_quality_pct: u8,
    pub is_connected: bool,
}

impl RadioMetrics {
    /// Create a new radio metrics snapshot.
    pub fn new(
        rssi_dbm: i16,
        snr_db: f32,
        tx_power_dbm: i16,
        link_quality_pct: u8,
        is_connected: bool,
    ) -> Self {
        Self {
            rssi_dbm,
            snr_db,
            tx_power_dbm,
            tx_packets: 0,
            rx_packets: 0,
            tx_bytes: 0,
            rx_bytes: 0,
            tx_errors: 0,
            rx_errors: 0,
            link_quality_pct,
            is_connected,
        }
    }
}

impl RadioMetrics {
    /// Calculate packet loss rate as a ratio (0.0 - 1.0).
    pub fn packet_loss_rate(&self) -> f32 {
        let total = self.tx_packets + self.tx_errors;
        if total == 0 {
            0.0
        } else {
            self.tx_errors as f32 / total as f32
        }
    }

    /// Check if radio link quality or signal is degraded.
    pub fn is_link_degraded(&self) -> bool {
        !self.is_connected || self.link_quality_pct < 40 || self.rssi_dbm < -90
    }
}

impl RadioMetrics {
    /// Record transmitted packet and byte count.
    pub fn record_tx(&mut self, bytes: u64, success: bool) {
        self.tx_bytes += bytes;
        if success {
            self.tx_packets += 1;
        } else {
            self.tx_errors += 1;
        }
    }

    /// Record received packet and byte count.
    pub fn record_rx(&mut self, bytes: u64, success: bool) {
        self.rx_bytes += bytes;
        if success {
            self.rx_packets += 1;
        } else {
            self.rx_errors += 1;
        }
    }
}

/// Heartbeat generator configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeartbeatConfig {
    pub uid: String,
    pub callsign: String,
    pub cot_type: String,
    pub interval_ms: u64,
    pub stale_duration_ms: u64,
}

impl HeartbeatConfig {
    /// Create a new heartbeat configuration with default interval.
    pub fn new(uid: impl Into<String>, callsign: impl Into<String>) -> Self {
        Self {
            uid: uid.into(),
            callsign: callsign.into(),
            cot_type: "a-f-A-M-F-Q".to_string(),
            interval_ms: 1000,
            stale_duration_ms: 3000,
        }
    }

    /// Set custom CoT type code.
    pub fn with_cot_type(mut self, cot_type: impl Into<String>) -> Self {
        self.cot_type = cot_type.into();
        self
    }

    /// Set custom timing intervals.
    pub fn with_intervals(mut self, interval_ms: u64, stale_ms: u64) -> Self {
        self.interval_ms = interval_ms;
        self.stale_duration_ms = stale_ms;
        self
    }
}

/// Snapshot container of an emitted heartbeat event.
#[derive(Debug, Clone, PartialEq)]
pub struct HeartbeatRecord {
    pub sequence: u64,
    pub timestamp_ms: u64,
    pub stale_ms: u64,
    pub platform: PlatformMetrics,
    pub radio: RadioMetrics,
    pub cot_xml: String,
}

/// Periodic airborne telemetry heartbeat generator.
#[derive(Debug, Clone)]
pub struct HeartbeatGenerator {
    config: HeartbeatConfig,
    sequence: u64,
    platform: PlatformMetrics,
    radio: RadioMetrics,
    last_generated_ms: Option<u64>,
    total_generated: u64,
}

impl HeartbeatGenerator {
    /// Create a new heartbeat generator instance.
    pub fn new(
        config: HeartbeatConfig,
        platform: PlatformMetrics,
        radio: RadioMetrics,
    ) -> Self {
        Self {
            config,
            sequence: 0,
            platform,
            radio,
            last_generated_ms: None,
            total_generated: 0,
        }
    }

    /// Update platform metrics.
    pub fn update_platform(&mut self, platform: PlatformMetrics) {
        self.platform = platform;
    }
}

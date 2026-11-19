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
    pub fn new(config: HeartbeatConfig, platform: PlatformMetrics, radio: RadioMetrics) -> Self {
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

impl HeartbeatGenerator {
    /// Update radio link metrics.
    pub fn update_radio(&mut self, radio: RadioMetrics) {
        self.radio = radio;
    }

    /// Update current platform pose coordinates.
    pub fn update_pose(&mut self, pose: Option<PlatformPose>) {
        self.platform.pose = pose;
    }

    /// Check if a heartbeat is due for emission.
    pub fn is_due(&self, now_ms: u64) -> bool {
        match self.last_generated_ms {
            None => true,
            Some(last) => now_ms.saturating_sub(last) >= self.config.interval_ms,
        }
    }
}

impl HeartbeatGenerator {
    /// Time remaining until next heartbeat is due in milliseconds.
    pub fn time_until_next_ms(&self, now_ms: u64) -> u64 {
        match self.last_generated_ms {
            None => 0,
            Some(last) => {
                let elapsed = now_ms.saturating_sub(last);
                self.config.interval_ms.saturating_sub(elapsed)
            }
        }
    }

    /// Render CoT point XML element based on platform pose.
    fn render_point_xml(&self) -> String {
        if let Some(ref p) = self.platform.pose {
            format!(
                r#"<point lat="{:.7}" lon="{:.7}" hae="{:.2}" ce="9999999" le="9999999"/>"#,
                p.lat_deg, p.lon_deg, p.hae_m
            )
        } else {
            r#"<point lat="0.0000000" lon="0.0000000" hae="0.00" ce="9999999" le="9999999"/>"#
                .to_string()
        }
    }
}

impl HeartbeatGenerator {
    /// Render detail XML tag with platform health and radio metrics.
    fn render_detail_xml(&self) -> String {
        format!(
            r#"<detail><contact callsign="{callsign}"/><status state="{state}" battery="{bat:.1}" v="{v:.2}" temp="{temp:.1}" cpu="{cpu:.1}" uptime="{up}"/><radio rssi="{rssi}" snr="{snr:.1}" pwr="{pwr}" tx_pkts="{tx_p}" rx_pkts="{rx_p}" tx_err="{tx_e}" rx_err="{rx_e}" q="{q}" conn="{conn}"/></detail>"#,
            callsign = escape_xml_attr(&self.platform.callsign),
            state = self.platform.state.as_str(),
            bat = self.platform.battery_pct,
            v = self.platform.battery_voltage_v,
            temp = self.platform.cpu_temp_c,
            cpu = self.platform.cpu_load_pct,
            up = self.platform.uptime_secs,
            rssi = self.radio.rssi_dbm,
            snr = self.radio.snr_db,
            pwr = self.radio.tx_power_dbm,
            tx_p = self.radio.tx_packets,
            rx_p = self.radio.rx_packets,
            tx_e = self.radio.tx_errors,
            rx_e = self.radio.rx_errors,
            q = self.radio.link_quality_pct,
            conn = if self.radio.is_connected { "1" } else { "0" },
        )
    }
}

impl HeartbeatGenerator {
    /// Format full Cursor-on-Target XML event string.
    pub fn render_cot_xml(&self, now_ms: u64, stale_ms: u64) -> String {
        let time_str = format_iso8601_utc(now_ms as i64);
        let stale_str = format_iso8601_utc(stale_ms as i64);
        let point = self.render_point_xml();
        let detail = self.render_detail_xml();
        format!(
            r#"<event version="2.0" uid="{uid}" type="{cot_type}" how="m-g" time="{t}" start="{t}" stale="{s}">{point}{detail}</event>"#,
            uid = escape_xml_attr(&self.config.uid),
            cot_type = escape_xml_attr(&self.config.cot_type),
            t = time_str,
            s = stale_str,
            point = point,
            detail = detail,
        )
    }
}

impl HeartbeatGenerator {
    /// Force generate a heartbeat record.
    pub fn generate(&mut self, now_ms: u64) -> HeartbeatRecord {
        self.sequence += 1;
        self.total_generated += 1;
        self.last_generated_ms = Some(now_ms);
        let stale_ms = now_ms + self.config.stale_duration_ms;
        let cot_xml = self.render_cot_xml(now_ms, stale_ms);
        HeartbeatRecord {
            sequence: self.sequence,
            timestamp_ms: now_ms,
            stale_ms,
            platform: self.platform.clone(),
            radio: self.radio.clone(),
            cot_xml,
        }
    }

    /// Generate heartbeat record only if due according to configured interval.
    pub fn generate_if_due(&mut self, now_ms: u64) -> Option<HeartbeatRecord> {
        if self.is_due(now_ms) {
            Some(self.generate(now_ms))
        } else {
            None
        }
    }
}

impl HeartbeatGenerator {
    /// Total count of heartbeats generated.
    pub const fn total_generated(&self) -> u64 {
        self.total_generated
    }

    /// Monotonic sequence number of last generated heartbeat.
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    /// Timestamp of last generated heartbeat in milliseconds.
    pub const fn last_generated_ms(&self) -> Option<u64> {
        self.last_generated_ms
    }

    /// Reference to configuration.
    pub const fn config(&self) -> &HeartbeatConfig {
        &self.config
    }

    /// Reset sequence counters and emission timestamps.
    pub fn reset(&mut self) {
        self.sequence = 0;
        self.last_generated_ms = None;
        self.total_generated = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_state_and_health() {
        let state = PlatformOperationalState::Nominal;
        assert_eq!(state.as_str(), "NOMINAL");
        assert!(state.is_healthy());

        let emergency = PlatformOperationalState::Emergency;
        assert_eq!(emergency.as_str(), "EMERGENCY");
        assert!(!emergency.is_healthy());

        let fault = PlatformOperationalState::Fault;
        assert_eq!(fault.as_str(), "FAULT");
        assert!(!fault.is_healthy());
    }

    #[test]
    fn test_platform_metrics_thresholds() {
        let mut m = PlatformMetrics::new(
            "UAV-01",
            PlatformOperationalState::Nominal,
            18.5,
            14.2,
            72.5,
            45.0,
            3600,
            None,
        );
        assert!(m.is_battery_low(20.0));
        assert!(!m.is_battery_low(15.0));
        assert!(m.is_cpu_overheating(70.0));
        assert!(!m.is_cpu_overheating(75.0));

        let pose = PlatformPose::new(100, 37.7749, -122.4194, 500.0, 0.0, 0.0, 1.57);
        m.set_pose(Some(pose));
        assert!(m.pose.is_some());
    }

    #[test]
    fn test_radio_metrics_and_counters() {
        let mut radio = RadioMetrics::new(-75, 18.2, 20, 85, true);
        assert!(!radio.is_link_degraded());
        assert_eq!(radio.packet_loss_rate(), 0.0);

        radio.record_tx(256, true);
        radio.record_tx(256, false);
        assert_eq!(radio.tx_packets, 1);
        assert_eq!(radio.tx_errors, 1);
        assert_eq!(radio.tx_bytes, 512);
        assert!((radio.packet_loss_rate() - 0.5).abs() < f32::EPSILON);

        radio.record_rx(128, true);
        assert_eq!(radio.rx_packets, 1);
        assert_eq!(radio.rx_bytes, 128);

        radio.is_connected = false;
        assert!(radio.is_link_degraded());
    }

    #[test]
    fn test_heartbeat_generator_timing_and_xml() {
        let cfg = HeartbeatConfig::new("UAV-ALPHA", "EAGLE-1").with_intervals(1000, 3000);
        let plat = PlatformMetrics::new(
            "EAGLE-1",
            PlatformOperationalState::Nominal,
            85.0,
            16.2,
            45.0,
            20.0,
            100,
            None,
        );
        let rad = RadioMetrics::new(-65, 22.0, 25, 95, true);
        let mut gen = HeartbeatGenerator::new(cfg, plat, rad);

        assert!(gen.is_due(0));
        assert_eq!(gen.time_until_next_ms(0), 0);

        let rec = gen.generate(1000);
        assert_eq!(rec.sequence, 1);
        assert_eq!(rec.timestamp_ms, 1000);
        assert_eq!(rec.stale_ms, 4000);
        assert!(rec.cot_xml.contains("uid=\"UAV-ALPHA\""));
        assert!(rec.cot_xml.contains("callsign=\"EAGLE-1\""));

        assert!(!gen.is_due(1500));
        assert_eq!(gen.time_until_next_ms(1500), 500);
        assert!(gen.is_due(2000));
        assert_eq!(gen.time_until_next_ms(2000), 0);
    }
}

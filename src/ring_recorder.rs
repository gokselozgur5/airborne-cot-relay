//! In-memory circular telemetry recorder with crash dump dumpers.

use crate::PlatformPose;
use std::collections::VecDeque;
use std::fmt;
use std::io::Write;
use std::sync::RwLock;

/// Error conditions occurring during telemetry crash dumping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CrashDumpError {
    /// IO error occurred while writing crash dump stream.
    Io(String),
    /// Formatting or serialization failure.
    Formatting(String),
    /// The recorder buffer is empty, no telemetry to dump.
    EmptyBuffer,
}

impl fmt::Display for CrashDumpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "crash dump io error: {}", e),
            Self::Formatting(e) => {
                write!(f, "crash dump formatting error: {}", e)
            }
            Self::EmptyBuffer => write!(f, "recorder buffer is empty"),
        }
    }
}

impl std::error::Error for CrashDumpError {}

/// Recorded platform telemetry sample with flight status metrics.
#[derive(Debug, Clone, PartialEq)]
pub struct TelemetryRecord {
    /// Monotonic sequence identifier.
    pub sequence: u64,
    /// Unix millisecond timestamp.
    pub timestamp_ms: u64,
    /// Platform spatial pose and attitude.
    pub pose: PlatformPose,
    /// System battery voltage in Volts.
    pub battery_volts: f32,
    /// Core processor temperature in Celsius.
    pub cpu_temp_c: f32,
    /// Datalink radio quality percentage (0-100).
    pub link_quality_pct: u8,
    /// Bitfield flags for operational alerts.
    pub error_flags: u32,
}

impl TelemetryRecord {
    /// Construct a new telemetry record.
    pub const fn new(
        sequence: u64,
        timestamp_ms: u64,
        pose: PlatformPose,
        battery_volts: f32,
        cpu_temp_c: f32,
        link_quality_pct: u8,
        error_flags: u32,
    ) -> Self {
        Self {
            sequence,
            timestamp_ms,
            pose,
            battery_volts,
            cpu_temp_c,
            link_quality_pct,
            error_flags,
        }
    }
}

/// Supported output formats for telemetry crash dump export.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DumpFormat {
    /// Structured JSON format with array of records.
    Json,
    /// Comma-separated values format with header row.
    Csv,
    /// Human-readable diagnostic text dump.
    Text,
}

/// Buffer eviction policy when recorder capacity is exceeded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverwriteMode {
    /// Overwrite oldest recorded telemetry samples.
    OverwriteOldest,
    /// Reject new telemetry records when capacity is full.
    RejectNew,
}

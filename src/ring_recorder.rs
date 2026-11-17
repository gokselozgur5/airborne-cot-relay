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

/// Configuration parameters for telemetry ring recorder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RingRecorderConfig {
    /// Maximum number of records retained in ring buffer.
    pub capacity: usize,
    /// Eviction policy when capacity is exhausted.
    pub overwrite_mode: OverwriteMode,
}

impl Default for RingRecorderConfig {
    fn default() -> Self {
        Self {
            capacity: 1024,
            overwrite_mode: OverwriteMode::OverwriteOldest,
        }
    }
}

/// Recorder runtime telemetry buffer metrics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RecorderStats {
    /// Cumulative count of samples accepted into recorder.
    pub total_recorded: u64,
    /// Total samples dropped due to capacity eviction.
    pub total_overwritten: u64,
    /// Maximum ring buffer capacity.
    pub buffer_capacity: usize,
    /// Current number of stored telemetry records.
    pub current_len: usize,
    /// Timestamp of oldest record in buffer, if any.
    pub oldest_timestamp_ms: Option<u64>,
    /// Timestamp of newest record in buffer, if any.
    pub newest_timestamp_ms: Option<u64>,
}

/// Forensic crash dump report containing metadata and telemetry trace.
#[derive(Debug, Clone, PartialEq)]
pub struct CrashDumpReport {
    /// Incident timestamp in milliseconds.
    pub timestamp_ms: u64,
    /// Human-readable crash or fault reason description.
    pub reason: String,
    /// Numeric platform fault code.
    pub fault_code: u32,
    /// Cumulative samples recorded prior to crash.
    pub total_recorded: u64,
    /// Number of records evicted prior to crash.
    pub total_overwritten: u64,
    /// Chronological list of recorded telemetry frames.
    pub records: Vec<TelemetryRecord>,
}

struct RecorderInner {
    buffer: VecDeque<TelemetryRecord>,
    total_recorded: u64,
    total_overwritten: u64,
}

/// Circular in-memory telemetry recorder with thread-safe access.
pub struct TelemetryRingRecorder {
    config: RingRecorderConfig,
    inner: RwLock<RecorderInner>,
}

impl TelemetryRingRecorder {
    /// Create a new recorder with default configuration and given capacity.
    pub fn new(capacity: usize) -> Self {
        Self::with_config(RingRecorderConfig {
            capacity: capacity.max(1),
            ..Default::default()
        })
    }

    /// Create a new recorder with custom configuration.
    pub fn with_config(config: RingRecorderConfig) -> Self {
        let capacity = config.capacity.max(1);
        Self {
            config: RingRecorderConfig { capacity, ..config },
            inner: RwLock::new(RecorderInner {
                buffer: VecDeque::with_capacity(capacity),
                total_recorded: 0,
                total_overwritten: 0,
            }),
        }
    }
}

impl TelemetryRingRecorder {
    /// Record a telemetry sample into the circular buffer.
    pub fn record(&self, record: TelemetryRecord) -> bool {
        let mut inner = match self.inner.write() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        if inner.buffer.len() >= self.config.capacity {
            match self.config.overwrite_mode {
                OverwriteMode::OverwriteOldest => {
                    inner.buffer.pop_front();
                    inner.total_overwritten += 1;
                }
                OverwriteMode::RejectNew => return false,
            }
        }
        inner.buffer.push_back(record);
        inner.total_recorded += 1;
        true
    }
}

impl TelemetryRingRecorder {
    /// Convenience helper to record a new pose with sensor metrics.
    pub fn record_pose(
        &self,
        sequence: u64,
        timestamp_ms: u64,
        pose: PlatformPose,
        battery_volts: f32,
        cpu_temp_c: f32,
        link_quality_pct: u8,
        error_flags: u32,
    ) -> bool {
        self.record(TelemetryRecord::new(
            sequence,
            timestamp_ms,
            pose,
            battery_volts,
            cpu_temp_c,
            link_quality_pct,
            error_flags,
        ))
    }
}

impl TelemetryRingRecorder {
    /// Clear all recorded telemetry and reset counters.
    pub fn clear(&self) {
        let mut inner = match self.inner.write() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        inner.buffer.clear();
        inner.total_recorded = 0;
        inner.total_overwritten = 0;
    }

    /// Number of active telemetry records currently in the buffer.
    pub fn len(&self) -> usize {
        let inner = match self.inner.read() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        inner.buffer.len()
    }
}

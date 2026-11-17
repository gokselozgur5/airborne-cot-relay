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

impl TelemetryRingRecorder {
    /// Check whether the ring recorder buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Maximum capacity of the ring buffer.
    pub fn capacity(&self) -> usize {
        self.config.capacity
    }

    /// Retrieve latest operational recorder statistics.
    pub fn stats(&self) -> RecorderStats {
        let inner = match self.inner.read() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        RecorderStats {
            total_recorded: inner.total_recorded,
            total_overwritten: inner.total_overwritten,
            buffer_capacity: self.config.capacity,
            current_len: inner.buffer.len(),
            oldest_timestamp_ms: inner.buffer.front().map(|r| r.timestamp_ms),
            newest_timestamp_ms: inner.buffer.back().map(|r| r.timestamp_ms),
        }
    }
}

impl TelemetryRingRecorder {
    /// Fetch the newest recorded telemetry sample.
    pub fn latest(&self) -> Option<TelemetryRecord> {
        let inner = match self.inner.read() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        inner.buffer.back().cloned()
    }

    /// Fetch the oldest recorded telemetry sample currently retained.
    pub fn oldest(&self) -> Option<TelemetryRecord> {
        let inner = match self.inner.read() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        inner.buffer.front().cloned()
    }
}

impl TelemetryRingRecorder {
    /// Extract a snapshot of all active records in chronological order.
    pub fn snapshot(&self) -> Vec<TelemetryRecord> {
        let inner = match self.inner.read() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        inner.buffer.iter().cloned().collect()
    }

    /// Return the most recent N records in chronological order.
    pub fn get_recent(&self, count: usize) -> Vec<TelemetryRecord> {
        let inner = match self.inner.read() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        let skip = inner.buffer.len().saturating_sub(count);
        inner.buffer.iter().skip(skip).cloned().collect()
    }
}

impl TelemetryRingRecorder {
    /// Return all records with timestamps within `[start_ms, end_ms]`.
    pub fn get_time_range(&self, start_ms: u64, end_ms: u64) -> Vec<TelemetryRecord> {
        let inner = match self.inner.read() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        inner
            .buffer
            .iter()
            .filter(|r| r.timestamp_ms >= start_ms && r.timestamp_ms <= end_ms)
            .cloned()
            .collect()
    }
}

impl TelemetryRingRecorder {
    /// Generate a forensic crash dump snapshot with cause and fault code.
    pub fn create_crash_dump(
        &self,
        reason: &str,
        fault_code: u32,
        timestamp_ms: u64,
    ) -> CrashDumpReport {
        let inner = match self.inner.read() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        CrashDumpReport {
            timestamp_ms,
            reason: reason.to_string(),
            fault_code,
            total_recorded: inner.total_recorded,
            total_overwritten: inner.total_overwritten,
            records: inner.buffer.iter().cloned().collect(),
        }
    }
}

impl TelemetryRingRecorder {
    /// Export a crash dump report into a string in the specified format.
    pub fn dump_to_string(
        &self,
        report: &CrashDumpReport,
        format: DumpFormat,
    ) -> Result<String, CrashDumpError> {
        let mut buffer = Vec::new();
        self.dump_to_writer(report, format, &mut buffer)?;
        String::from_utf8(buffer).map_err(|e| CrashDumpError::Formatting(e.to_string()))
    }

    /// Write crash dump report to an arbitrary output stream.
    pub fn dump_to_writer<W: Write>(
        &self,
        report: &CrashDumpReport,
        format: DumpFormat,
        writer: &mut W,
    ) -> Result<(), CrashDumpError> {
        match format {
            DumpFormat::Json => self.write_json(report, writer),
            DumpFormat::Csv => self.write_csv(report, writer),
            DumpFormat::Text => self.write_text(report, writer),
        }
    }
}

impl TelemetryRingRecorder {
    fn format_csv_row(r: &TelemetryRecord) -> String {
        format!(
            "{},{},{:.6},{:.6},{:.2},{:.4},{:.4},{:.4},{:.2},{:.1},{},{}\n",
            r.sequence,
            r.timestamp_ms,
            r.pose.lat_deg,
            r.pose.lon_deg,
            r.pose.hae_m,
            r.pose.roll_rad,
            r.pose.pitch_rad,
            r.pose.yaw_rad,
            r.battery_volts,
            r.cpu_temp_c,
            r.link_quality_pct,
            r.error_flags,
        )
    }
}

impl TelemetryRingRecorder {
    fn write_csv<W: Write>(
        &self,
        report: &CrashDumpReport,
        writer: &mut W,
    ) -> Result<(), CrashDumpError> {
        writeln!(
            writer,
            "seq,timestamp_ms,lat,lon,hae,roll,pitch,yaw,battery_v,cpu_c,link_pct,err_flags"
        )
        .map_err(|e| CrashDumpError::Io(e.to_string()))?;
        for r in &report.records {
            writer
                .write_all(Self::format_csv_row(r).as_bytes())
                .map_err(|e| CrashDumpError::Io(e.to_string()))?;
        }
        Ok(())
    }
}

impl TelemetryRingRecorder {
    fn format_json_record(r: &TelemetryRecord) -> String {
        format!(
            "{{\"seq\":{},\"time_ms\":{},\"lat\":{:.6},\"lon\":{:.6},\"hae\":{:.2},\"roll\":{:.4},\"pitch\":{:.4},\"yaw\":{:.4},\"bat\":{:.2},\"temp\":{:.1},\"link\":{},\"flags\":{}}}",
            r.sequence,
            r.timestamp_ms,
            r.pose.lat_deg,
            r.pose.lon_deg,
            r.pose.hae_m,
            r.pose.roll_rad,
            r.pose.pitch_rad,
            r.pose.yaw_rad,
            r.battery_volts,
            r.cpu_temp_c,
            r.link_quality_pct,
            r.error_flags,
        )
    }
}

impl TelemetryRingRecorder {
    fn write_json_header<W: Write>(
        report: &CrashDumpReport,
        writer: &mut W,
    ) -> Result<(), CrashDumpError> {
        let esc = report.reason.replace('"', "\\\"");
        write!(
            writer,
            "{{\"timestamp_ms\":{},\"reason\":\"{}\",\"fault_code\":{},\"total_recorded\":{},\"total_overwritten\":{},\"records\":[",
            report.timestamp_ms, esc, report.fault_code,
            report.total_recorded, report.total_overwritten,
        ).map_err(|e| CrashDumpError::Io(e.to_string()))
    }
}

impl TelemetryRingRecorder {
    fn write_json<W: Write>(
        &self,
        report: &CrashDumpReport,
        writer: &mut W,
    ) -> Result<(), CrashDumpError> {
        Self::write_json_header(report, writer)?;
        for (i, r) in report.records.iter().enumerate() {
            if i > 0 {
                write!(writer, ",").map_err(|e| CrashDumpError::Io(e.to_string()))?;
            }
            writer
                .write_all(Self::format_json_record(r).as_bytes())
                .map_err(|e| CrashDumpError::Io(e.to_string()))?;
        }
        write!(writer, "]}}").map_err(|e| CrashDumpError::Io(e.to_string()))?;
        Ok(())
    }
}

impl TelemetryRingRecorder {
    fn format_text_header(report: &CrashDumpReport) -> String {
        format!(
            "=== AIRBORNE COT RELAY CRASH DUMP ===\nTimestamp:    {} ms\nFault Code:   0x{:08X}\nCrash Reason: {}\nRecorded:     {} (overwritten: {})\nTrace Entries: {}\n--- TELEMETRY TRACE ---\n",
            report.timestamp_ms,
            report.fault_code,
            report.reason,
            report.total_recorded,
            report.total_overwritten,
            report.records.len(),
        )
    }
}

impl TelemetryRingRecorder {
    fn write_text<W: Write>(
        &self,
        report: &CrashDumpReport,
        writer: &mut W,
    ) -> Result<(), CrashDumpError> {
        writer
            .write_all(Self::format_text_header(report).as_bytes())
            .map_err(|e| CrashDumpError::Io(e.to_string()))?;
        for r in &report.records {
            writeln!(
                writer,
                "#{:05} @ {:>8}ms | lat={:8.4} lon={:8.4} alt={:6.1}m | bat={:4.1}V temp={:4.1}C link={:3}%",
                r.sequence,
                r.timestamp_ms,
                r.pose.lat_deg,
                r.pose.lon_deg,
                r.pose.hae_m,
                r.battery_volts,
                r.cpu_temp_c,
                r.link_quality_pct
            )
            .map_err(|e| CrashDumpError::Io(e.to_string()))?;
        }
        Ok(())
    }
}

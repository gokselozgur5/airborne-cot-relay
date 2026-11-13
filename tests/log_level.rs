use airborne_cot_relay::log::{Level, Logger, ParseLevelError, Record};
use std::str::FromStr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

struct TestLogger {
    level: Level,
    calls: AtomicUsize,
}

impl Logger for TestLogger {
    fn level(&self) -> Level {
        self.level
    }

    fn log(&self, record: &Record<'_>) {
        if self.enabled(record.level) {
            self.calls.fetch_add(1, Ordering::SeqCst);
        }
    }
}

#[test]
fn test_level_ordering() {
    assert!(Level::Error < Level::Warn);
    assert!(Level::Warn < Level::Info);
    assert!(Level::Info < Level::Debug);
    assert!(Level::Debug < Level::Trace);
}

#[test]
fn test_level_display_and_as_str() {
    assert_eq!(Level::Error.as_str(), "error");
    assert_eq!(Level::Warn.as_str(), "warn");
    assert_eq!(Level::Info.as_str(), "info");
    assert_eq!(Level::Debug.as_str(), "debug");
    assert_eq!(Level::Trace.as_str(), "trace");
    assert_eq!(format!("{}", Level::Info), "info");
}

#[test]
fn test_level_from_str_case_insensitive() {
    assert_eq!(Level::from_str("error").unwrap(), Level::Error);
    assert_eq!(Level::from_str("ERROR").unwrap(), Level::Error);
    assert_eq!("Warn".parse::<Level>().unwrap(), Level::Warn);
    assert_eq!("warning".parse::<Level>().unwrap(), Level::Warn);
    assert_eq!("INFO".parse::<Level>().unwrap(), Level::Info);
    assert_eq!("Debug".parse::<Level>().unwrap(), Level::Debug);
    assert_eq!("TRACE".parse::<Level>().unwrap(), Level::Trace);
}

#[test]
fn test_level_from_str_invalid() {
    let err = "unknown".parse::<Level>().unwrap_err();
    assert_eq!(err, ParseLevelError("unknown".to_string()));
    assert!(format!("{}", err).contains("invalid log level"));
}

#[test]
fn test_record_fields_and_lookup() {
    let fields: [(&str, &str); 2] = [("key1", "val1"), ("key2", "val2")];
    let record = Record::new(1000, Level::Info, "relay", "msg", &fields);
    assert_eq!(record.ts_ms, 1000);
    assert_eq!(record.level, Level::Info);
    assert_eq!(record.target, "relay");
    assert_eq!(record.message, "msg");
    assert_eq!(record.get_field("key1"), Some("val1"));
    assert_eq!(record.get_field("missing"), None);
}

#[test]
fn test_logger_threshold_filtering() {
    let logger = TestLogger {
        level: Level::Info,
        calls: AtomicUsize::new(0),
    };
    assert!(logger.enabled(Level::Error));
    assert!(logger.enabled(Level::Warn));
    assert!(logger.enabled(Level::Info));
    assert!(!logger.enabled(Level::Debug));
    assert!(!logger.enabled(Level::Trace));

    let err_rec = Record::new(1, Level::Error, "tgt", "err", &[]);
    let dbg_rec = Record::new(2, Level::Debug, "tgt", "dbg", &[]);
    logger.log(&err_rec);
    logger.log(&dbg_rec);
    assert_eq!(logger.calls.load(Ordering::SeqCst), 1);
}

#[test]
fn test_logger_reference_and_arc() {
    let logger = Arc::new(TestLogger {
        level: Level::Debug,
        calls: AtomicUsize::new(0),
    });
    let rec = Record::new(1, Level::Debug, "tgt", "msg", &[]);

    let logger_ref = &*logger;
    assert!(logger_ref.enabled(Level::Debug));
    logger_ref.log(&rec);

    assert!(logger.enabled(Level::Debug));
    logger.log(&rec);
    assert_eq!(logger.calls.load(Ordering::SeqCst), 2);
}

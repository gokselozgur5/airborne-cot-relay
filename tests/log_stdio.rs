use airborne_cot_relay::log::stdio::{format_record, StdioLogger};
use airborne_cot_relay::log::{Level, Logger, Record};

#[test]
fn test_format_record_uncolored() {
    let fields = [("key", "val"), ("mode", "live")];
    let record = Record::new(1234, Level::Info, "network", "client connected", &fields);
    let rendered = format_record(&record, false);
    assert_eq!(
        rendered,
        "[1234] INFO [network] client connected key=val mode=live"
    );
}

#[test]
fn test_format_record_without_fields() {
    let record = Record::new(5678, Level::Warn, "tracker", "lost target", &[]);
    let rendered = format_record(&record, false);
    assert_eq!(rendered, "[5678] WARN [tracker] lost target");
}

#[test]
fn test_format_record_colored() {
    let record = Record::new(100, Level::Error, "core", "critical error", &[]);
    let rendered = format_record(&record, true);
    assert!(rendered.contains("\x1b[31mERROR\x1b[0m"));
    assert!(rendered.starts_with("[100] "));
    assert!(rendered.ends_with(" critical error"));

    let debug_rec = Record::new(101, Level::Debug, "core", "debug info", &[]);
    let dbg_rendered = format_record(&debug_rec, true);
    assert!(dbg_rendered.contains("\x1b[34mDEBUG\x1b[0m"));
}

#[test]
fn test_stdio_logger_properties_and_filtering() {
    let logger = StdioLogger::new(Level::Info, true);
    assert_eq!(logger.level(), Level::Info);
    assert!(logger.is_colored());
    assert!(logger.enabled(Level::Error));
    assert!(logger.enabled(Level::Info));
    assert!(!logger.enabled(Level::Debug));

    let record = Record::new(999, Level::Info, "app", "boot", &[]);
    let formatted = logger.format(&record);
    assert!(formatted.contains("\x1b[32mINFO\x1b[0m"));

    logger.log(&record);
    let debug_record = Record::new(1000, Level::Debug, "app", "hidden", &[]);
    logger.log(&debug_record);
}

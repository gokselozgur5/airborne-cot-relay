use airborne_cot_relay::log::stdio::{format_record, StdioLogger};
use airborne_cot_relay::log::{Level, Logger, Record};

#[test]
fn test_format_record_uncolored() {
    let fields = [("key", "val"), ("mode", "live")];
    let record = Record::new(1234, Level::Info, "network", "client connected", &fields);
    let rendered = format_record(&record, false);
    assert_eq!(rendered, "[1234] INFO [network] client connected key=val mode=live");
}

#[test]
fn test_format_record_without_fields() {
    let record = Record::new(5678, Level::Warn, "tracker", "lost target", &[]);
    let rendered = format_record(&record, false);
    assert_eq!(rendered, "[5678] WARN [tracker] lost target");
}

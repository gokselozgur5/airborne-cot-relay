use airborne_cot_relay::log::{format_json, JsonLogger, Level, Logger, Record};
use std::sync::{Arc, Mutex};

#[derive(Default, Clone)]
struct BufferWriter(Arc<Mutex<Vec<u8>>>);

impl std::io::Write for BufferWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn test_format_json_and_escaping() {
    let rec = Record::new(
        1234,
        Level::Info,
        "app\"core",
        "hello\n\t\\world",
        &[("key", "val\"quote")],
    );
    let s = format_json(&rec);
    assert!(s.contains("\"ts\":1234"));
    assert!(s.contains("\"target\":\"app\\\"core\""));
    assert!(s.contains("\"message\":\"hello\\n\\t\\\\world\""));
    assert!(s.contains("\"key\":\"val\\\"quote\""));
}

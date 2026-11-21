use airborne_cot_relay::log::stdio::format_record;
use airborne_cot_relay::log::{format_json, Level, Record};

#[test]
fn json_line_has_no_raw_control_characters_or_duplicate_keys() {
    let fields = [("level", "error"), ("k", "a\u{1b}[31m\u{0}b")];
    let record = Record::new(1, Level::Info, "relay", "bell\u{7}", &fields);
    let line = format_json(&record);
    let raw: Vec<u32> = line.chars().map(u32::from).filter(|c| *c < 0x20).collect();
    assert!(raw.is_empty(), "raw control characters {raw:?} in {line:?}");
    assert_eq!(line.matches("\"level\":").count(), 1, "{line:?}");
}

#[test]
fn stdio_line_escapes_line_breaks_and_carriage_returns() {
    let fields = [("note", "line1\nline2\rline3")];
    let record = Record::new(1, Level::Info, "relay", "forged\nsecond line", &fields);
    let rendered = format_record(&record, false);
    assert!(
        !rendered.contains('\n'),
        "rendered line has unescaped newline: {rendered:?}"
    );
    assert!(
        !rendered.contains('\r'),
        "rendered line has unescaped carriage return: {rendered:?}"
    );
    assert!(rendered.contains("forged\\nsecond line"));
    assert!(rendered.contains("note=line1\\nline2\\rline3"));
}

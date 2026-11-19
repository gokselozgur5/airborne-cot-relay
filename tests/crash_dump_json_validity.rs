use airborne_cot_relay::{DumpFormat, PlatformPose, TelemetryRingRecorder};

#[test]
fn json_crash_dump_is_valid_json() {
    let recorder = TelemetryRingRecorder::new(4);
    let pose = PlatformPose::new(1000, 34.0, -118.0, 100.0, 0.0, 0.0, 0.0);
    recorder.record_pose(1, 100, pose, f32::NAN, 40.0, 90, 0);
    let report = recorder.create_crash_dump("panic at src\\main.rs\nline 2", 1, 2000);
    let json = recorder.dump_to_string(&report, DumpFormat::Json).unwrap();
    assert!(!json.contains("NaN"), "NaN is not a JSON value: {json}");
    assert!(
        json.contains(r#"src\\main.rs\nline 2"#),
        "reason is not escaped: {json}"
    );
}

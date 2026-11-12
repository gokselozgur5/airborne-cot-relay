use std::process::Command;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_airborne-cot-relay"))
        .args(args)
        .output()
        .unwrap()
}

fn stdout(args: &[&str]) -> String {
    String::from_utf8_lossy(&run(args).stdout).into()
}
#[test]
fn dry_run_defaults_to_mock() {
    assert!(stdout(&["--dry-run"]).contains("dry-run: mode=mock"));
}

#[test]
fn dry_run_resolves_live_in_either_order() {
    assert!(stdout(&["--dry-run", "--live"]).contains("dry-run: mode=live"));
    assert!(stdout(&["--live", "--dry-run"]).contains("dry-run: mode=live"));
}

#[test]
fn dry_run_skips_mission_start() {
    let out = run(&["--dry-run", "--live"]);
    assert!(out.status.success());
    assert!(!String::from_utf8_lossy(&out.stdout).contains("Listening"));
}

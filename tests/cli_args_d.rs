use std::process::Command;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_airborne-cot-relay"))
        .args(args)
        .output()
        .unwrap()
}

fn help_text() -> String {
    String::from_utf8_lossy(&run(&["--help"]).stdout).into()
}

#[test]
fn help_lists_every_option() {
    let text = help_text();
    assert!(text.contains("Usage:"));
    for opt in ["--live", "--mock", "--help", "--version", "--dry-run"] {
        assert!(text.contains(opt), "missing {opt}");
    }
}

#[test]
fn help_omits_banner_and_exits_zero() {
    let out = run(&["-h"]);
    assert!(out.status.success());
    assert!(!String::from_utf8_lossy(&out.stdout).contains("Airborne CoT Relay v"));
}

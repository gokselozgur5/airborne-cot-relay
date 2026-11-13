use std::process::{Command, Output};

/// Runs the binary with `arg` and a single environment override.
fn run_with_env(arg: &str, key: &str, value: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_airborne-cot-relay"))
        .arg(arg)
        .env(key, value)
        .output()
        .expect("binary runs")
}

fn assert_config_error(out: &Output) {
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("Invalid config"), "stderr: {stderr}");
}

#[test]
fn malformed_cot_address_exits_2() {
    assert_config_error(&run_with_env("--mock", "RELAY_COT_ADDR", "nonsense"));
}

#[test]
fn non_numeric_stale_timeout_exits_2_in_live_mode() {
    let out = run_with_env("--live", "RELAY_STALE_TIMEOUT_MS", "abc");
    assert_config_error(&out);
}

#[test]
fn help_ignores_an_invalid_environment() {
    let out = run_with_env("--help", "RELAY_COT_ADDR", "nonsense");
    assert_eq!(out.status.code(), Some(0));
}

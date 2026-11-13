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

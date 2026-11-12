use std::process::Command;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_airborne-cot-relay"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn unknown_argument_exits_two() {
    let out = run(&["--lvie"]);
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn unknown_argument_is_named_on_stderr() {
    let err = String::from_utf8_lossy(&run(&["--lvie"]).stderr).to_string();
    assert!(err.contains("error: unrecognized argument '--lvie'"));
}

#[test]
fn known_flags_still_exit_zero() {
    for flag in ["--live", "-m", "--help"] {
        assert_eq!(run(&[flag]).status.code(), Some(0));
    }
}

use std::process::Command;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_airborne-cot-relay"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn version_flag_prints_package_version() {
    let out = run(&["--version"]);
    let text = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        text.trim(),
        concat!("airborne-cot-relay ", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn short_version_flag_exits_zero() {
    assert_eq!(run(&["-V"]).status.code(), Some(0));
}

use std::net::UdpSocket;
use std::process::Command;

#[test]
fn live_mode_fails_when_the_telemetry_address_is_already_bound() {
    let taken = UdpSocket::bind("127.0.0.1:0").unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_airborne-cot-relay"))
        .arg("--live")
        .env(
            "RELAY_TELEMETRY_ADDR",
            taken.local_addr().unwrap().to_string(),
        )
        .env("RELAY_COT_ADDR", "127.0.0.1:6969")
        .output()
        .expect("binary runs");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_ne!(out.status.code(), Some(0), "stdout: {stdout}");
}

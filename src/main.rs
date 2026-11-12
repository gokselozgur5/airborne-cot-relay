use std::env;
use std::process::ExitCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissionMode {
    Live,
    Mock,
    Help,
}

impl MissionMode {
    pub fn parse_args<I: Iterator<Item = String>>(mut args: I) -> Self {
        let _bin = args.next();
        match args.next().as_deref() {
            Some("--live") | Some("-l") => Self::Live,
            Some("--mock") | Some("-m") => Self::Mock,
            Some("--help") | Some("-h") => Self::Help,
            _ => Self::Mock,
        }
    }
}

fn print_banner() {
    println!("Airborne CoT Relay v0.1.0");
    println!("Sensor-to-TAK Cursor-on-Target relay runner");
}

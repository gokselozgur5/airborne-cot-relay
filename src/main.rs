use airborne_cot_relay::app::config_boot::{boot_config_from_env, CONFIG_ERROR_EXIT};
use std::env;
use std::process::ExitCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissionMode {
    Live,
    Mock,
    Help,
    Version,
}

impl MissionMode {
    pub fn parse_args<I: Iterator<Item = String>>(mut args: I) -> Self {
        let _bin = args.next();
        match args.next().as_deref() {
            Some("--live") | Some("-l") => Self::Live,
            Some("--mock") | Some("-m") => Self::Mock,
            Some("--help") | Some("-h") => Self::Help,
            Some("--version") | Some("-V") => Self::Version,
            _ => Self::Mock,
        }
    }
}

const KNOWN_FLAGS: [&str; 8] = [
    "--live",
    "-l",
    "--mock",
    "-m",
    "--help",
    "-h",
    "--version",
    "-V",
];

fn find_unknown_arg(args: &[String]) -> Option<&String> {
    args.iter()
        .skip(1)
        .find(|a| !KNOWN_FLAGS.contains(&a.as_str()))
}

fn print_banner() {
    println!("Airborne CoT Relay v0.1.0");
    println!("Sensor-to-TAK Cursor-on-Target relay runner");
}

fn run_mission(mode: MissionMode) -> ExitCode {
    match mode {
        MissionMode::Help => {
            println!(
                "Usage: airborne-cot-relay [OPTIONS]
"
            );
            println!("  -l, --live     Use live telemetry and camera streams");
            println!("  -m, --mock     Run a simulated flight path (default)");
            println!("  -h, --help     Print this help text");
            println!("  -V, --version  Print the version");
            println!("      --dry-run  Resolve the mode and exit");
            ExitCode::SUCCESS
        }
        MissionMode::Version => {
            println!("airborne-cot-relay {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        MissionMode::Mock => {
            println!("Starting simulated 60-second flight path...");
            ExitCode::SUCCESS
        }
        MissionMode::Live => {
            println!("Listening for live telemetry and camera streams...");
            ExitCode::SUCCESS
        }
    }
}

fn main() -> ExitCode {
    let mut args: Vec<String> = env::args().collect();
    let dry_run = args.iter().any(|a| a == "--dry-run");
    args.retain(|a| a != "--dry-run");
    if let Some(bad) = find_unknown_arg(&args) {
        eprintln!("error: unrecognized argument '{bad}'");
        eprintln!("Usage: airborne-cot-relay [--live | --mock | --help]");
        return ExitCode::from(2);
    }
    let mode = MissionMode::parse_args(args.into_iter());
    if !matches!(mode, MissionMode::Version | MissionMode::Help) {
        print_banner();
    }
    if dry_run {
        println!("dry-run: mode={}", format!("{mode:?}").to_lowercase());
        return ExitCode::SUCCESS;
    }
    if matches!(mode, MissionMode::Mock | MissionMode::Live) {
        if let Err(code) = boot_or_exit() {
            return code;
        }
    }
    run_mission(mode)
}

fn boot_or_exit() -> Result<airborne_cot_relay::RelayConfig, ExitCode> {
    boot_config_from_env().map_err(|e| {
        eprintln!("error: {e}");
        ExitCode::from(CONFIG_ERROR_EXIT)
    })
}

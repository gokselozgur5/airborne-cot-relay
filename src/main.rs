use std::env;
use std::process::ExitCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissionMode {
    Live,
    Mock,
    Help,
}

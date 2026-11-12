//! Throughput timing walkthrough.
//!
//! Runs the targeter over a fixed number of frames with one detection per
//! frame and reports wall-clock frames per second. Use `--release` for
//! representative numbers.

use airborne_cot_relay::{PlatformPose, RelayConfig, TargetDetection, Targeter};
use std::time::Instant;

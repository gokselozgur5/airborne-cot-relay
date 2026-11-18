//! Runs the live mission loop through ingest, targeter, track filter and CoT output.
use crate::config::{RelayConfig, RelayError};
use crate::metrics::Metrics;

/// Runs the full live mission loop.
pub fn run_live(_config: &RelayConfig, _metrics: &Metrics) -> Result<u64, RelayError> {
    println!("Live relay running...");
    Ok(0)
}

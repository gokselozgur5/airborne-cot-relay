//! Runs the live mission loop through ingest, targeter, track filter and CoT output.
use crate::config::{RelayConfig, RelayError};
use crate::metrics::Metrics;

/// Runs the full live mission loop.
pub fn run_live(config: &RelayConfig, metrics: &Metrics) -> Result<u64, RelayError> {
    println!("Starting live mission relay loop...");
    // TODO: Connect ingest, targeter, track filter and relay_loop here.
    Ok(0)
}

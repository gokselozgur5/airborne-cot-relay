//! Relay pipeline wiring telemetry ingest, targeting and output stages.

use std::sync::Arc;

use crate::metrics::Metrics;
use crate::{RelayConfig, RelayError, Targeter, TelemetryIngest};

/// Library pipeline built from a validated relay configuration.
pub struct Pipeline {
    config: RelayConfig,
    ingest: TelemetryIngest,
    targeter: Targeter,
    metrics: Arc<Metrics>,
}

impl Pipeline {
    /// Validates `config` and builds every stage from it, with the NED
    /// origin at latitude and longitude zero (see [`Pipeline::with_origin`]).
    pub fn from_config(config: &RelayConfig) -> Result<Self, RelayError> {
        config.validate()?;
        let alt = config.ground_elevation_m;
        Ok(Self {
            config: config.clone(),
            ingest: TelemetryIngest::new(0.0, 0.0, alt, config.target_stale_timeout_ms),
            targeter: Targeter::new(&config.camera, alt),
            metrics: Arc::new(Metrics::new()),
        })
    }
}

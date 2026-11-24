//! Relay pipeline wiring telemetry ingest, targeting and output stages.

mod telemetry_stage;

use std::sync::Arc;

use crate::metrics::Metrics;
use crate::{RelayConfig, RelayError, Targeter, TelemetryIngest};

pub mod output_stage;
pub mod summary;

/// Library pipeline built from a validated relay configuration.
pub struct Pipeline {
    config: RelayConfig,
    ingest: TelemetryIngest,
    targeter: Targeter,
    metrics: Arc<Metrics>,
    stale_gate: crate::runtime::stale_gate::StaleGate,
}

impl Pipeline {
    /// Validates `config` and builds every stage from it, with the NED
    /// origin at latitude and longitude zero (see [`Pipeline::with_origin`]).
    pub fn from_config(config: &RelayConfig) -> Result<Self, RelayError> {
        config.validate()?;
        let alt = config.ground_elevation_m;
        let stale_ms = config.target_stale_timeout_ms;
        Ok(Self {
            config: config.clone(),
            ingest: TelemetryIngest::new(0.0, 0.0, alt, stale_ms),
            targeter: Targeter::new(&config.camera, alt),
            metrics: Arc::new(Metrics::new()),
            stale_gate: crate::runtime::stale_gate::StaleGate::new(stale_ms),
        })
    }
}

impl TryFrom<&RelayConfig> for Pipeline {
    type Error = RelayError;

    fn try_from(config: &RelayConfig) -> Result<Self, Self::Error> {
        Self::from_config(config)
    }
}

impl TryFrom<RelayConfig> for Pipeline {
    type Error = RelayError;

    fn try_from(config: RelayConfig) -> Result<Self, Self::Error> {
        Self::from_config(&config)
    }
}

impl std::fmt::Debug for Pipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pipeline")
            .field("config", &self.config)
            .field("metrics", &self.metrics)
            .finish()
    }
}

impl Pipeline {
    /// Configuration the pipeline was built from.
    pub fn config(&self) -> &RelayConfig {
        &self.config
    }

    /// Shared metrics handle updated by every stage.
    pub fn metrics(&self) -> &Arc<Metrics> {
        &self.metrics
    }

    /// Latest pose decoded by the telemetry stage, if any.
    pub fn latest_pose(&self) -> Option<crate::PlatformPose> {
        self.ingest.latest_pose()
    }

    /// Ground-projection targeter used by the detection stage.
    pub fn targeter(&self) -> &Targeter {
        &self.targeter
    }

    /// Telemetry freshness gate controlling outbound CoT sending.
    pub fn stale_gate(&self) -> &crate::runtime::stale_gate::StaleGate {
        &self.stale_gate
    }

    /// Mutable telemetry freshness gate.
    pub fn stale_gate_mut(&mut self) -> &mut crate::runtime::stale_gate::StaleGate {
        &mut self.stale_gate
    }
}

impl Pipeline {
    /// Replaces the telemetry NED origin used for geodetic conversion.
    ///
    /// Any buffered telemetry bytes and the latest pose are discarded.
    pub fn with_origin(mut self, lat_deg: f64, lon_deg: f64, alt_m: f64) -> Self {
        let max_age_ms = self.config.target_stale_timeout_ms;
        self.ingest = TelemetryIngest::new(lat_deg, lon_deg, alt_m, max_age_ms);
        self.stale_gate = crate::runtime::stale_gate::StaleGate::new(max_age_ms);
        self
    }
}

mod detect_stage;

pub mod queue_stage;

pub mod step;

pub mod events;
pub use events::{EventTracker, PipelineEvent};
pub mod stats;

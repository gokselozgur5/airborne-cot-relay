//! Sink recovery coordinator driving ReconnectLoop on degradation.

use super::reconnect::ReconnectLoop;
use super::sleep::Sleeper;
use crate::config::RelayError;
use crate::pipeline::events::PipelineEvent;

/// Coordinates sink reconnection when degraded.
#[derive(Debug)]
pub struct SinkRecovery<S: Sleeper> {
    reconnect_loop: ReconnectLoop<S>,
    reconnects: u64,
}

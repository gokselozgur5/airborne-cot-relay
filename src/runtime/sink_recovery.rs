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

impl<S: Sleeper> SinkRecovery<S> {
    /// Creates a new SinkRecovery instance.
    pub fn new(reconnect_loop: ReconnectLoop<S>) -> Self {
        Self {
            reconnect_loop,
            reconnects: 0,
        }
    }

    /// Access the underlying reconnect loop.
    pub fn reconnect_loop(&self) -> &ReconnectLoop<S> {
        &self.reconnect_loop
    }

    /// Number of successful reconnections.
    pub fn reconnects(&self) -> u64 {
        self.reconnects
    }
}

impl<S: Sleeper> SinkRecovery<S> {
    /// Handles a pipeline event. If `SinkDegraded`, triggers ReconnectLoop.
    pub fn on_event<T, F>(
        &mut self,
        event: &PipelineEvent,
        connect: F,
    ) -> Result<Option<T>, RelayError>
    where
        F: FnMut() -> Result<T, RelayError>,
    {
        match event {
            PipelineEvent::SinkDegraded { .. } => {
                let sink = self.reconnect_loop.run(connect)?;
                self.reconnects = self.reconnects.saturating_add(1);
                Ok(Some(sink))
            }
            _ => Ok(None),
        }
    }
}

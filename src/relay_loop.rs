//! Reconnecting relay loop combining backoff, queue, and health tracking.

use crate::config::RelayError;
use crate::metrics::Metrics;
use crate::relay::{Backoff, BoundedQueue, Clock, CotSink, DropPolicy, SendHealth};
use std::sync::Arc;

/// Reconnecting send loop that drains a bounded queue into a recoverable sink.
pub struct ReconnectingRelayLoop<S, C, F> {
    sink: S,
    factory: F,
    queue: BoundedQueue<Vec<u8>>,
    health: SendHealth,
    backoff: Backoff,
    clock: C,
    metrics: Arc<Metrics>,
    reconnect_deadline_ms: Option<i64>,
}

impl<S: CotSink, C: Clock, F: FnMut() -> Result<S, RelayError>> ReconnectingRelayLoop<S, C, F> {
    /// Creates a new reconnecting loop with given components.
    pub fn new(
        sink: S,
        factory: F,
        capacity: usize,
        policy: DropPolicy,
        clock: C,
        metrics: Arc<Metrics>,
    ) -> Self {
        let queue = BoundedQueue::new(capacity, policy);
        metrics.set_queue_depth(0);
        Self {
            sink,
            factory,
            queue,
            health: SendHealth::new(3),
            backoff: Backoff::new(50, 2000),
            clock,
            metrics,
            reconnect_deadline_ms: None,
        }
    }
}

impl<S: CotSink, C: Clock, F: FnMut() -> Result<S, RelayError>> ReconnectingRelayLoop<S, C, F> {
    /// Enqueues a payload and updates queue metrics.
    pub fn enqueue(&mut self, payload: Vec<u8>) -> Option<Vec<u8>> {
        let dropped = self.queue.push(payload);
        if dropped.is_some() {
            self.metrics.add_dropped(1);
        }
        self.metrics.set_queue_depth(self.queue.len() as u64);
        dropped
    }
}

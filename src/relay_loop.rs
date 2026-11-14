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

impl<S: CotSink, C: Clock, F: FnMut() -> Result<S, RelayError>> ReconnectingRelayLoop<S, C, F> {
    fn try_reconnect(&mut self) {
        let now = self.clock.now_ms();
        if let Some(deadline) = self.reconnect_deadline_ms {
            if now < deadline {
                return;
            }
        }
        self.metrics.add_reconnect();
        match (self.factory)() {
            Ok(new_sink) => {
                self.sink = new_sink;
                self.health = SendHealth::new(3);
                self.backoff.reset();
                self.reconnect_deadline_ms = None;
            }
            Err(_) => {
                let delay = self.backoff.next_delay_ms();
                self.reconnect_deadline_ms = Some(now.saturating_add(delay as i64));
            }
        }
    }
}

impl<S: CotSink, C: Clock, F: FnMut() -> Result<S, RelayError>> ReconnectingRelayLoop<S, C, F> {
    fn send_one(&mut self, payload: &[u8]) -> Result<usize, RelayError> {
        let res = self.sink.send(payload);
        if let Ok(sent) = res {
            if sent == payload.len() {
                self.health.record(&Ok(sent));
                self.metrics.record_sent(self.clock.now_ms() as u64);
                return Ok(sent);
            }
        }
        self.health.record(&res);
        res
    }
}

impl<S: CotSink, C: Clock, F: FnMut() -> Result<S, RelayError>> ReconnectingRelayLoop<S, C, F> {
    /// Polls the send loop to drain pending payloads or trigger reconnection.
    pub fn poll(&mut self) -> Result<(), RelayError> {
        if self.health.needs_reconnect() {
            self.try_reconnect();
            if self.health.needs_reconnect() {
                return Ok(());
            }
        }
        while let Some(payload) = self.queue.pop() {
            match self.send_one(&payload) {
                Ok(_) => continue,
                Err(RelayError::IoError(ref m)) if m.contains("WouldBlock") => {
                    self.queue.push(payload);
                    break;
                }
                Err(_) => break,
            }
        }
        self.metrics.set_queue_depth(self.queue.len() as u64);
        Ok(())
    }
}

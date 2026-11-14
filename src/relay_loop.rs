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

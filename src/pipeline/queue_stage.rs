//! Bounded outbound queue between the targeter and the sink.

use crate::metrics::Metrics;
use crate::relay::{BoundedQueue, DropPolicy};

/// Outbound queue stage that reports overflow drops and depth to `Metrics`.
#[derive(Debug)]
pub struct QueueStage<T> {
    queue: BoundedQueue<T>,
}

impl<T> QueueStage<T> {
    /// Creates a stage holding at most `capacity` events.
    pub fn new(capacity: usize, policy: DropPolicy) -> Self {
        Self {
            queue: BoundedQueue::new(capacity, policy),
        }
    }
}

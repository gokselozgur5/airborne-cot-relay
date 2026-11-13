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

impl<T> QueueStage<T> {
    /// Enqueues one event, counting any overflow drop and updating the depth gauge.
    ///
    /// Returns the event that was discarded on overflow, if any.
    pub fn push(&mut self, item: T, metrics: &Metrics) -> Option<T> {
        let before = self.queue.dropped();
        let discarded = self.queue.push(item);
        metrics.add_dropped(self.queue.dropped().saturating_sub(before));
        self.publish_depth(metrics);
        discarded
    }

    /// Dequeues the oldest event and updates the depth gauge.
    pub fn pop(&mut self, metrics: &Metrics) -> Option<T> {
        let item = self.queue.pop();
        self.publish_depth(metrics);
        item
    }

    fn publish_depth(&self, metrics: &Metrics) {
        metrics.set_queue_depth(self.queue.len() as u64);
    }
}

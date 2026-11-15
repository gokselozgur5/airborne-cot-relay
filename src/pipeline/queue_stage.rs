//! Bounded outbound queue between the targeter and the sink.

use crate::metrics::Metrics;
use crate::relay::{BoundedQueue, DropPolicy};

/// Outbound queue stage that reports overflow drops and depth to `Metrics`.
#[derive(Debug)]
pub struct QueueStage<T> {
    queue: BoundedQueue<T>,
    capacity: usize,
    policy: DropPolicy,
}

impl<T> QueueStage<T> {
    /// Creates a stage holding at most `capacity` events.
    pub fn new(capacity: usize, policy: DropPolicy) -> Self {
        let capacity = capacity.max(1);
        Self {
            queue: BoundedQueue::new(capacity, policy),
            capacity,
            policy,
        }
    }

    /// Maximum capacity of the stage.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Drop policy applied when the stage is full.
    pub fn policy(&self) -> DropPolicy {
        self.policy
    }

    /// True when the stage is at capacity and backpressure drops will occur.
    pub fn is_full(&self) -> bool {
        self.len() >= self.capacity
    }

    /// Remaining slots available before backpressure drops occur.
    pub fn remaining_capacity(&self) -> usize {
        self.capacity.saturating_sub(self.len())
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

impl<T> QueueStage<T> {
    /// Number of queued events.
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// True when no events are queued.
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Total overflow drops since the stage was created.
    pub fn dropped(&self) -> u64 {
        self.queue.dropped()
    }
}

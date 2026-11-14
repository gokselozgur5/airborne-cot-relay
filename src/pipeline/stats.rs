/// Pipeline packet statistics tracker.
#[derive(Debug, Default, Clone)]
pub struct PipelineStats {
    received: u64,
    dropped: u64,
}

impl PipelineStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn received(&self) -> u64 {
        self.received
    }

    pub fn dropped(&self) -> u64 {
        self.dropped
    }
}

impl PipelineStats {
    pub fn drop_rate(&self) -> f64 {
        if self.received == 0 {
            0.0
        } else {
            self.dropped as f64 / self.received as f64
        }
    }
}

impl PipelineStats {
    pub fn record_received(&mut self, count: u64) {
        self.received = self.received.saturating_add(count);
    }

    pub fn record_dropped(&mut self, count: u64) {
        self.dropped = self.dropped.saturating_add(count);
    }
}

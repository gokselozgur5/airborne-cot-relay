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

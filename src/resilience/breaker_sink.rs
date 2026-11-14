use std::error::Error;

/// A wrapper around a CotSink that implements open breaker accounting.
pub struct BreakerSink<S> {
    inner: S,
    rejected_count: usize,
}

impl<S: Send + Sync + 'static> BreakerSink<S> {
    /// Creates a new BreakerSink wrapping the given CotSink.
    pub fn new(inner: S) -> Self {
        BreakerSink {
            inner,
            rejected_count: 0,
        }
    }

    /// Sends data through the inner sink. If the breaker is open, returns an error without calling the inner sink and counts each rejected send.
    pub async fn send(&mut self, data: S::Item) -> Result<(), Box<dyn Error>> {
        if self.is_open() {
            self.rejected_count += 1;
            Err(Box::new(std::io::Error::from(std::io::ErrorKind::Other, "Breaker is open")))
        } else {
            self.inner.send(data).await

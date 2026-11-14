use std::error::Error;

mod common;

#[tokio::test]
async fn test_breaker_sink() {
    use super::*;
    use common::{CotSink, MockCotSink};

    let mut mock_cot_sink = MockCotSink::new();
    let mut breaker_sink = BreakerSink::new(mock_cot_sink.clone());

    // Simulate sending data through the inner sink
    assert_eq!(breaker_sink.send("Hello").await, Err(Box::new(std::io::Error::from(std::io::ErrorKind::Other, "Breaker is open"))));

    // Check if the rejected count is incremented
    assert_eq!(mock_cot_sink.rejected_count(), 1);

    // Reset the rejected count and send data again
    breaker_sink.reset_rejected_count();
    assert_eq!(breaker_sink.send("Hello").await, Ok(()));

    // Check if the rejected count is reset to zero
    assert_eq!(mock_cot_sink.rejected_count(), 0);
}

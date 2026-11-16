//! TCP auto-reconnecting stream sender with exponential backoff.

use crate::config::RelayError;
use crate::relay::CotSink;
use std::collections::VecDeque;
use std::io::{self, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::{Duration, Instant};

/// Errors encountered during reconnecting client operations.
#[derive(Debug)]
pub enum ReconnectClientError {
    Io(io::Error),
    BackoffActive { remaining_ms: u64 },
    MaxRetriesExceeded { attempts: usize },
    QueueFull { current: usize, max: usize },
    NotConnected,
}

impl std::fmt::Display for ReconnectClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "TCP client I/O error: {}", e),
            Self::BackoffActive { remaining_ms } => {
                write!(f, "reconnect backoff active ({}ms remaining)", remaining_ms)
            }
            Self::MaxRetriesExceeded { attempts } => {
                write!(f, "max reconnection retries exceeded: {}", attempts)
            }
            Self::QueueFull { current, max } => {
                write!(f, "send queue full ({}/{})", current, max)
            }
            Self::NotConnected => write!(f, "TCP client is not connected"),
        }
    }
}

impl std::error::Error for ReconnectClientError {}

impl From<io::Error> for ReconnectClientError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<ReconnectClientError> for RelayError {
    fn from(err: ReconnectClientError) -> Self {
        RelayError::IoError(err.to_string())
    }
}

/// Configuration policy governing reconnection behavior and queuing.
#[derive(Debug, Clone, PartialEq)]
pub struct ReconnectPolicy {
    pub initial_backoff: Duration,
    pub max_backoff: Duration,
    pub backoff_multiplier: f64,
    pub max_retries: Option<usize>,
    pub connect_timeout: Duration,
    pub max_queue_messages: usize,
    pub max_queue_bytes: usize,
    pub drop_oldest_on_full: bool,
}

impl Default for ReconnectPolicy {
    fn default() -> Self {
        Self {
            initial_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            max_retries: None,
            connect_timeout: Duration::from_secs(1),
            max_queue_messages: 1024,
            max_queue_bytes: 1024 * 1024,
            drop_oldest_on_full: true,
        }
    }
}

impl ReconnectPolicy {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_initial_backoff(mut self, backoff: Duration) -> Self {
        self.initial_backoff = backoff;
        self
    }

    pub fn with_max_backoff(mut self, backoff: Duration) -> Self {
        self.max_backoff = backoff;
        self
    }

    pub fn with_multiplier(mut self, multiplier: f64) -> Self {
        self.backoff_multiplier = multiplier.max(1.0);
        self
    }

    pub fn with_max_retries(mut self, retries: Option<usize>) -> Self {
        self.max_retries = retries;
        self
    }

    pub fn with_connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    pub fn with_queue_limits(mut self, max_messages: usize, max_bytes: usize) -> Self {
        self.max_queue_messages = max_messages;
        self.max_queue_bytes = max_bytes;
        self
    }

    pub fn with_drop_oldest(mut self, drop_oldest: bool) -> Self {
        self.drop_oldest_on_full = drop_oldest;
        self
    }
}

impl ReconnectPolicy {
    /// Computes backoff duration for given consecutive failure count.
    pub fn compute_backoff(&self, consecutive_failures: usize) -> Duration {
        if consecutive_failures == 0 {
            return self.initial_backoff;
        }
        let factor = self.backoff_multiplier.powi(consecutive_failures as i32);
        let computed_secs = self.initial_backoff.as_secs_f64() * factor;
        let candidate = Duration::from_secs_f64(computed_secs);
        candidate.min(self.max_backoff)
    }
}

/// Transmission and reconnection runtime metrics.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct StreamClientStats {
    pub connect_attempts: usize,
    pub successful_connects: usize,
    pub disconnections: usize,
    pub messages_enqueued: usize,
    pub messages_sent: usize,
    pub bytes_sent: usize,
    pub messages_dropped: usize,
    pub bytes_dropped: usize,
}

/// Resilient TCP client stream sender with automatic backoff reconnection.
#[derive(Debug)]
pub struct ReconnectClient {
    target_addr: SocketAddr,
    policy: ReconnectPolicy,
    stream: Option<TcpStream>,
    consecutive_failures: usize,
    next_connect_deadline: Option<Instant>,
    queue: VecDeque<Vec<u8>>,
    queued_bytes: usize,
    stats: StreamClientStats,
}

impl ReconnectClient {
    /// Creates a new ReconnectClient targeting given address with custom policy.
    pub fn new(target_addr: SocketAddr, policy: ReconnectPolicy) -> Self {
        Self {
            target_addr,
            policy,
            stream: None,
            consecutive_failures: 0,
            next_connect_deadline: None,
            queue: VecDeque::new(),
            queued_bytes: 0,
            stats: StreamClientStats::default(),
        }
    }

    /// Creates a new ReconnectClient targeting given address with default policy.
    pub fn with_defaults(target_addr: SocketAddr) -> Self {
        Self::new(target_addr, ReconnectPolicy::default())
    }
}

impl ReconnectClient {
    pub fn target_addr(&self) -> SocketAddr {
        self.target_addr
    }

    pub fn policy(&self) -> &ReconnectPolicy {
        &self.policy
    }

    pub fn is_connected(&self) -> bool {
        self.stream.is_some()
    }

    pub fn consecutive_failures(&self) -> usize {
        self.consecutive_failures
    }

    pub fn stats(&self) -> &StreamClientStats {
        &self.stats
    }
}

impl ReconnectClient {
    pub fn queued_messages(&self) -> usize {
        self.queue.len()
    }

    pub fn queued_bytes(&self) -> usize {
        self.queued_bytes
    }

    pub fn reset_backoff(&mut self) {
        self.consecutive_failures = 0;
        self.next_connect_deadline = None;
    }

    pub fn disconnect(&mut self) {
        if self.stream.take().is_some() {
            self.stats.disconnections += 1;
        }
    }
}

impl ReconnectClient {
    fn check_can_connect(&self, now: Instant) -> Result<(), ReconnectClientError> {
        if let Some(max) = self.policy.max_retries {
            if self.consecutive_failures >= max {
                return Err(ReconnectClientError::MaxRetriesExceeded {
                    attempts: self.consecutive_failures,
                });
            }
        }
        if let Some(deadline) = self.next_connect_deadline {
            if now < deadline {
                let ms = deadline.saturating_duration_since(now).as_millis() as u64;
                return Err(ReconnectClientError::BackoffActive { remaining_ms: ms });
            }
        }
        Ok(())
    }
}

impl ReconnectClient {
    fn handle_connect_failure(&mut self, now: Instant) {
        let backoff = self.policy.compute_backoff(self.consecutive_failures);
        self.consecutive_failures += 1;
        self.next_connect_deadline = Some(now + backoff);
        self.stream = None;
    }

    fn handle_connect_success(&mut self, stream: TcpStream) {
        self.stream = Some(stream);
        self.consecutive_failures = 0;
        self.next_connect_deadline = None;
        self.stats.successful_connects += 1;
    }
}

impl ReconnectClient {
    /// Attempts to connect to target address if not already connected.
    pub fn try_connect(&mut self) -> Result<bool, ReconnectClientError> {
        if self.stream.is_some() {
            return Ok(true);
        }
        let now = Instant::now();
        self.check_can_connect(now)?;
        self.stats.connect_attempts += 1;
        match TcpStream::connect_timeout(&self.target_addr, self.policy.connect_timeout) {
            Ok(s) => {
                self.handle_connect_success(s);
                Ok(true)
            }
            Err(e) => {
                self.handle_connect_failure(now);
                Err(ReconnectClientError::Io(e))
            }
        }
    }
}

impl ReconnectClient {
    fn drop_oldest_message(&mut self) {
        if let Some(dropped) = self.queue.pop_front() {
            self.queued_bytes = self.queued_bytes.saturating_sub(dropped.len());
            self.stats.messages_dropped += 1;
            self.stats.bytes_dropped += dropped.len();
        }
    }
}

impl ReconnectClient {
    /// Enqueues a payload for delivery, dropping oldest if configured on capacity.
    pub fn enqueue(&mut self, payload: Vec<u8>) -> Result<(), ReconnectClientError> {
        let msg_len = payload.len();
        while self.queue.len() >= self.policy.max_queue_messages
            || (self.queued_bytes + msg_len > self.policy.max_queue_bytes && !self.queue.is_empty())
        {
            if self.policy.drop_oldest_on_full {
                self.drop_oldest_message();
            } else {
                return Err(ReconnectClientError::QueueFull {
                    current: self.queue.len(),
                    max: self.policy.max_queue_messages,
                });
            }
        }
        self.queued_bytes += msg_len;
        self.queue.push_back(payload);
        self.stats.messages_enqueued += 1;
        Ok(())
    }
}

impl ReconnectClient {
    /// Flushes all queued messages over the active or newly-established connection.
    pub fn flush(&mut self) -> Result<usize, ReconnectClientError> {
        self.try_connect()?;
        let stream = match self.stream.as_mut() {
            Some(s) => s,
            None => return Err(ReconnectClientError::NotConnected),
        };
        let mut total_sent = 0usize;
        while let Some(msg) = self.queue.pop_front() {
            match stream.write_all(&msg) {
                Ok(()) => {
                    self.queued_bytes = self.queued_bytes.saturating_sub(msg.len());
                    self.stats.messages_sent += 1;
                    self.stats.bytes_sent += msg.len();
                    total_sent += msg.len();
                }
                Err(err) => {
                    self.queue.push_front(msg);
                    self.handle_connect_failure(Instant::now());
                    return Err(ReconnectClientError::Io(err));
                }
            }
        }
        Ok(total_sent)
    }
}

impl ReconnectClient {
    /// Attempts immediate transmission of payload over connected stream.
    pub fn send_immediate(&mut self, payload: &[u8]) -> Result<usize, ReconnectClientError> {
        self.try_connect()?;
        let stream = match self.stream.as_mut() {
            Some(s) => s,
            None => return Err(ReconnectClientError::NotConnected),
        };
        match stream.write_all(payload) {
            Ok(()) => {
                self.stats.messages_sent += 1;
                self.stats.bytes_sent += payload.len();
                Ok(payload.len())
            }
            Err(err) => {
                self.handle_connect_failure(Instant::now());
                Err(ReconnectClientError::Io(err))
            }
        }
    }
}

impl CotSink for ReconnectClient {
    fn send(&mut self, payload: &[u8]) -> Result<usize, RelayError> {
        match self.send_immediate(payload) {
            Ok(bytes) => Ok(bytes),
            Err(e) => {
                let _ = self.enqueue(payload.to_vec());
                Err(RelayError::from(e))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use std::net::TcpListener;

    #[test]
    fn test_reconnect_policy_defaults_and_builders() {
        let p = ReconnectPolicy::new()
            .with_initial_backoff(Duration::from_millis(50))
            .with_max_backoff(Duration::from_millis(500))
            .with_multiplier(3.0)
            .with_max_retries(Some(5))
            .with_queue_limits(10, 100)
            .with_drop_oldest(false);

        assert_eq!(p.initial_backoff, Duration::from_millis(50));
        assert_eq!(p.max_backoff, Duration::from_millis(500));
        assert!((p.backoff_multiplier - 3.0).abs() < f64::EPSILON);
        assert_eq!(p.max_retries, Some(5));
        assert_eq!(p.max_queue_messages, 10);
        assert_eq!(p.max_queue_bytes, 100);
        assert!(!p.drop_oldest_on_full);
    }

    #[test]
    fn test_backoff_exponential_growth_and_cap() {
        let p = ReconnectPolicy::new()
            .with_initial_backoff(Duration::from_millis(100))
            .with_max_backoff(Duration::from_millis(1000))
            .with_multiplier(2.0);

        assert_eq!(p.compute_backoff(0), Duration::from_millis(100));
        assert_eq!(p.compute_backoff(1), Duration::from_millis(200));
        assert_eq!(p.compute_backoff(2), Duration::from_millis(400));
        assert_eq!(p.compute_backoff(3), Duration::from_millis(800));
        assert_eq!(p.compute_backoff(4), Duration::from_millis(1000));
        assert_eq!(p.compute_backoff(10), Duration::from_millis(1000));
    }

    #[test]
    fn test_max_retries_exceeded_error() {
        let addr = "127.0.0.1:9".parse().unwrap();
        let p = ReconnectPolicy::new().with_max_retries(Some(2));
        let mut client = ReconnectClient::new(addr, p);
        client.consecutive_failures = 2;
        let err = client.try_connect().unwrap_err();
        match err {
            ReconnectClientError::MaxRetriesExceeded { attempts } => {
                assert_eq!(attempts, 2);
            }
            other => panic!("expected MaxRetriesExceeded, got {:?}", other),
        }
    }

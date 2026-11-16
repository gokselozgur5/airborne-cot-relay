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

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

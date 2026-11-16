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

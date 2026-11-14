use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use crate::output::Backoff;

/// A TCP client sink that connects on demand and uses backoff for reconnects.
pub struct TcpClientSink {
    stream: Option<TcpStream>,
    backoff: Backoff,
}

impl TcpClientSink {
    /// Creates a new `TcpClientSink` with the specified backoff schedule.
    pub fn builder(backoff: Backoff) -> TcpClientSinkBuilder {
        TcpClientSinkBuilder backoff
    }

    /// Connects to the target TCP server and returns a `TcpStream`.
    async fn connect(&mut self, addr: &ToSocketAddrs) -> Result<TcpStream, std::io::Error> {
        loop {
            match TcpStream::connect(addr).await {
                Ok(stream) => {
                    self.stream = Some(stream);
                    return Ok(stream);
                }
                Err(e) => {
                    if let Some(backoff) = &mut self.backoff {
                        backoff.sleep().await;
                    } else {
                        break; // No backoff, give up
                    }
                }
            }
        }
        Err(std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "Failed to connect"))
    }

    /// Sends a payload over the TCP stream.
    pub async fn send(&mut self, payload: &[u8]) -> Result<(), std::io::Error> {
        if let Some(stream) = &self.stream {
            stream.write_all(payload).await?;
            Ok(())
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "No connection established"))
        }
    }

    /// Closes the TCP stream.
    pub async fn close(&mut self) -> Result<(), std::io::Error> {
        if let Some(stream) = &self.stream {
            stream.close().await?;
            self.stream = None;
            Ok(())
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "No connection established"))
        }
    }
}

/// A builder for `TcpClientSink`.
pub struct TcpClientSinkBuilder {
    backoff: Backoff,
}

impl TcpClientSinkBuilder {
    /// Sets the backoff schedule.
    pub fn with_backoff(mut self, backoff: Backoff) -> Self {
        self.backoff = backoff;
        self
    }

    /// Builds a `TcpClientSink`.

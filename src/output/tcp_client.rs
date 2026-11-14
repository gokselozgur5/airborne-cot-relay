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

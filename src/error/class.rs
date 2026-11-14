//! Classification of `RelayError` by how a loop should react to it.

/// How a relay loop should react to an error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorClass {
    /// The process cannot continue; stop and report.
    Fatal,
    /// A transient failure; retry with backoff.
    Retryable,
    /// A bad input item; drop it and keep running.
    Droppable,
}

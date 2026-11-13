//! INI-style configuration lexer.

use std::fmt;

/// An error encountered while lexing an INI file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexerError {
    pub line: usize,
    pub message: String,
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for LexerError {}

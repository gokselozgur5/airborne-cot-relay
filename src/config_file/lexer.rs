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

/// A parsed key-value entry from an INI file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IniEntry {
    pub section: String,
    pub key: String,
    pub value: String,
    pub line: usize,
}

/// A parsed INI section holding entries in insertion order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IniSection {
    pub name: String,
    pub line: usize,
    pub entries: Vec<IniEntry>,
}

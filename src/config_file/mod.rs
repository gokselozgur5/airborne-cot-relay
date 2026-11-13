//! INI-style configuration file parser and lexer.

pub mod lexer;

pub use lexer::{IniEntry, IniSection, LexerError, ParsedIni};

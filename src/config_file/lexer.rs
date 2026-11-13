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

impl IniSection {
    pub fn new(name: impl Into<String>, line: usize) -> Self {
        Self {
            name: name.into(),
            line,
            entries: Vec::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|e| e.key == key)
            .map(|e| e.value.as_str())
    }
}

/// A fully parsed INI document.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParsedIni {
    pub sections: Vec<IniSection>,
}

impl ParsedIni {
    pub fn section(&self, name: &str) -> Option<&IniSection> {
        self.sections.iter().find(|s| s.name == name)
    }

    pub fn get(&self, section: &str, key: &str) -> Option<&str> {
        self.section(section)?.get(key)
    }

    pub fn entries(&self) -> impl Iterator<Item = &IniEntry> {
        self.sections.iter().flat_map(|s| s.entries.iter())
    }
}

fn parse_section_name(s: &str, line: usize) -> Result<String, LexerError> {
    if !s.ends_with(']') {
        return Err(LexerError { line, message: "unclosed section header".into() });
    }
    let name = s[1..s.len() - 1].trim();
    if name.is_empty() {
        return Err(LexerError { line, message: "empty section name".into() });
    }
    Ok(name.to_string())
}

fn parse_key_value(s: &str, line: usize) -> Result<(&str, &str), LexerError> {
    let eq = s.find('=').ok_or_else(|| LexerError {
        line,
        message: "missing '=' in key-value pair".into(),
    })?;
    let key = s[..eq].trim();
    if key.is_empty() {
        return Err(LexerError { line, message: "empty key".into() });
    }
    Ok((key, s[eq + 1..].trim()))
}

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
    /// Parses an INI formatted string into structured sections and key-values.
    pub fn parse(input: &str) -> Result<Self, LexerError> {
        let mut state = LexerState::default();
        for (i, raw) in input.lines().enumerate() {
            let line = i + 1;
            let s = raw.trim();
            if s.is_empty() || s.starts_with('#') {
                continue;
            }
            if s.starts_with('[') {
                let name = parse_section_name(s, line)?;
                state.set_section(&name, line);
            } else {
                let (k, v) = parse_key_value(s, line)?;
                state.add_entry(k, v, line)?;
            }
        }
        Ok(Self {
            sections: state.sections,
        })
    }

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
        return Err(LexerError {
            line,
            message: "unclosed section header".into(),
        });
    }
    let name = s[1..s.len() - 1].trim();
    if name.is_empty() {
        return Err(LexerError {
            line,
            message: "empty section name".into(),
        });
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
        return Err(LexerError {
            line,
            message: "empty key".into(),
        });
    }
    Ok((key, s[eq + 1..].trim()))
}

#[derive(Default)]
struct LexerState {
    sections: Vec<IniSection>,
    seen: std::collections::HashSet<(String, String)>,
    cur: Option<usize>,
}

impl LexerState {
    fn set_section(&mut self, name: &str, line: usize) {
        let idx = match self.sections.iter().position(|s| s.name == name) {
            Some(i) => i,
            None => {
                self.sections.push(IniSection::new(name, line));
                self.sections.len() - 1
            }
        };
        self.cur = Some(idx);
    }

    fn add_entry(&mut self, k: &str, v: &str, line: usize) -> Result<(), LexerError> {
        let idx = self.cur.unwrap_or_else(|| {
            self.sections.push(IniSection::new("", line));
            self.sections.len() - 1
        });
        self.cur = Some(idx);
        let sec = self.sections[idx].name.clone();
        if !self.seen.insert((sec.clone(), k.to_string())) {
            return Err(LexerError {
                line,
                message: format!("duplicate key '{k}' in section '{sec}'"),
            });
        }
        self.sections[idx].entries.push(IniEntry {
            section: sec,
            key: k.into(),
            value: v.into(),
            line,
        });
        Ok(())
    }
}

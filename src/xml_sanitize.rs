//! XML CoT sanitize and entity escaping utilities.

/// Returns true if character is valid under XML 1.0 specification.
pub fn is_xml_valid_char(c: char) -> bool {
    matches!(
        c as u32,
        0x9 | 0xA | 0xD | 0x20..=0xD7FF | 0xE000..=0xFFFD | 0x10000..=0x10FFFF
    )
}

/// Escapes standard XML predefined entity characters.
pub fn escape_xml_entity(c: char) -> Option<&'static str> {
    match c {
        '&' => Some("&amp;"),
        '<' => Some("&lt;"),
        '>' => Some("&gt;"),
        '"' => Some("&quot;"),
        '\'' => Some("&apos;"),
        '\t' => Some("&#9;"),
        '\n' => Some("&#10;"),
        '\r' => Some("&#13;"),
        _ => None,
    }
}

/// Strips characters that are not valid under XML 1.0.
pub fn sanitize_xml_chars(input: &str) -> String {
    input.chars().filter(|&c| is_xml_valid_char(c)).collect()
}

/// Escapes XML attribute special characters (&, <, >, ", ').
pub fn escape_xml_attr(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        if let Some(entity) = escape_xml_entity(c) {
            out.push_str(entity);
        } else {
            out.push(c);
        }
    }
    out
}

/// Escapes XML element text special characters (&, <, >).
pub fn escape_xml_text(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
    }
    out
}

/// Returns true if the string contains any characters requiring XML escaping.
pub fn has_xml_entities(input: &str) -> bool {
    input.chars().any(|c| escape_xml_entity(c).is_some())
}

/// Sanitizes invalid XML characters and escapes attribute entities in a single pass.
pub fn sanitize_and_escape_attr(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        if !is_xml_valid_char(c) {
            continue;
        }
        if let Some(entity) = escape_xml_entity(c) {
            out.push_str(entity);
        } else {
            out.push(c);
        }
    }
    out
}

/// Sanitizes invalid XML characters and escapes text entities in a single pass.
pub fn sanitize_and_escape_text(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        if !is_xml_valid_char(c) {
            continue;
        }
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
    }
    out
}

/// Sanitizes and escapes a Cursor-on-Target UID string.
pub fn sanitize_cot_uid(uid: &str) -> String {
    sanitize_and_escape_attr(uid.trim())
}

/// Sanitizes and escapes a Cursor-on-Target callsign string.
pub fn sanitize_cot_callsign(callsign: &str) -> String {
    sanitize_and_escape_attr(callsign.trim())
}

/// Restores predefined XML entity references back into original characters.
pub fn unescape_xml(input: &str) -> String {
    input
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_and_invalid_xml_chars() {
        assert!(is_xml_valid_char('a'));
        assert!(is_xml_valid_char('\t'));
        assert!(is_xml_valid_char('\n'));
        assert!(is_xml_valid_char('\r'));
        assert!(!is_xml_valid_char('\0'));
        assert!(!is_xml_valid_char('\x08'));
        assert!(!is_xml_valid_char('\x1b'));
    }

    #[test]
    fn test_escape_xml_attr_predefined_entities() {
        let raw = "<a & b 'c' \"d\">";
        let escaped = escape_xml_attr(raw);
        assert_eq!(escaped, "&lt;a &amp; b &apos;c&apos; &quot;d&quot;&gt;");
    }

    #[test]
    fn test_sanitize_and_escape_attr() {
        let dirty = "CALLSIGN\x00\x1b <VIP&FAST> \"#1\"";
        let clean = sanitize_and_escape_attr(dirty);
        assert_eq!(clean, "CALLSIGN &lt;VIP&amp;FAST&gt; &quot;#1&quot;");
    }

    #[test]
    fn test_unescape_and_roundtrip() {
        let original = "A & B < C > D 'E' \"F\"";
        let escaped = escape_xml_attr(original);
        assert_eq!(unescape_xml(&escaped), original);
    }
}

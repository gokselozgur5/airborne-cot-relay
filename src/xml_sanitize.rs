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

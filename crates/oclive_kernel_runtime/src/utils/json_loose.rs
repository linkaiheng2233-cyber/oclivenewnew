//! Extract a JSON object substring from model output for `serde_json` parsing (tolerant of surrounding noise / Markdown).

/// Take the slice from the first `{` to the last `}`; if it is not a valid JSON substring, the caller handles the parse failure.
#[must_use]
pub fn extract_json_object(raw: &str) -> Option<&str> {
    let start = raw.find('{')?;
    let end = raw.rfind('}')?;
    if end <= start {
        return None;
    }
    Some(&raw[start..=end])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_inner_object() {
        let s = r#"thought: ok
{"a":1}
tail"#;
        assert_eq!(extract_json_object(s), Some(r#"{"a":1}"#));
    }

    #[test]
    fn no_brace_returns_none() {
        assert_eq!(extract_json_object("no json"), None);
    }
}

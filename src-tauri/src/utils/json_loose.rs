//! 从模型输出中截取 JSON 对象子串，供 `serde_json` 解析（容忍前后废话/ Markdown）。

/// 取第一个 `{` 到最后一个 `}` 之间的切片；若不是合法 JSON 子串，由调用方解析失败处理。
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

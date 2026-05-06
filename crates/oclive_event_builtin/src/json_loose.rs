//! 从模型输出中截取 JSON 对象子串（与 runtime `utils::json_loose` 行为一致）。

/// 取第一个 `{` 到最后一个 `}` 之间的切片。
pub fn extract_json_object(raw: &str) -> Option<&str> {
    let start = raw.find('{')?;
    let end = raw.rfind('}')?;
    if end <= start {
        return None;
    }
    Some(&raw[start..=end])
}

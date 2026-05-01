use chrono::{DateTime, Utc};

/// 格式化 unix 时间戳为日期字符串 "YYYY-MM-DD"
pub fn format_timestamp_date(ts: i64) -> String {
    DateTime::<Utc>::from_timestamp(ts, 0)
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "1970-01-01".to_string())
}

/// 格式化 unix 时间戳为时间字符串 "HH:MM"
pub fn format_timestamp_time(ts: i64) -> String {
    DateTime::<Utc>::from_timestamp(ts, 0)
        .map(|dt| dt.format("%H:%M").to_string())
        .unwrap_or_else(|| "00:00".to_string())
}

/// 字符串截断，按字符数安全截断防止越界
pub fn truncate_str(s: &str, max_chars: usize) -> String {
    if s.chars().count() > max_chars {
        s.chars().take(max_chars).collect::<String>() + "..."
    } else {
        s.to_string()
    }
}

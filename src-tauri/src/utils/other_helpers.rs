use chrono::{DateTime, Utc};

/// Format a unix timestamp as a date string "YYYY-MM-DD".
#[must_use]
pub fn format_timestamp_date(ts: i64) -> String {
    DateTime::<Utc>::from_timestamp(ts, 0)
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "1970-01-01".to_string())
}

/// Format a unix timestamp as a time string "HH:MM".
#[must_use]
pub fn format_timestamp_time(ts: i64) -> String {
    DateTime::<Utc>::from_timestamp(ts, 0)
        .map(|dt| dt.format("%H:%M").to_string())
        .unwrap_or_else(|| "00:00".to_string())
}

/// Truncate a string safely by character count to avoid boundary overruns.
#[must_use]
pub fn truncate_str(s: &str, max_chars: usize) -> String {
    if s.chars().count() > max_chars {
        s.chars().take(max_chars).collect::<String>() + "..."
    } else {
        s.to_string()
    }
}

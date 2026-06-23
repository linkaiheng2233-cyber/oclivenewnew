//! Shared helpers for chat storage backends.

use chrono::{DateTime, Utc};

pub fn normalize_scene_id(scene_id: &str) -> String {
    let t = scene_id.trim();
    if t.is_empty() {
        "default".to_string()
    } else {
        t.to_string()
    }
}

pub fn timestamp_ms_to_rfc3339(ms: i64) -> String {
    if let Some(dt) = DateTime::from_timestamp_millis(ms) {
        return dt.to_rfc3339();
    }
    Utc::now().to_rfc3339()
}

pub fn cap_limit(limit: u32) -> u32 {
    if limit == 0 {
        u32::MAX
    } else {
        limit.min(10_000)
    }
}

pub fn rows_to_stored(rows: Vec<super::db::MessageRow>) -> Vec<super::types::StoredMessage> {
    rows.into_iter()
        .map(|r| super::types::StoredMessage {
            id: r.id,
            session_id: r.session_id,
            turn_index: r.turn_index,
            sender: r.sender,
            content: r.content,
            metadata: r.metadata,
            created_at: r.created_at,
        })
        .collect()
}

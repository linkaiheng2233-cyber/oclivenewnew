//! Shared types and helpers for `chat_sessions` / `chat_messages` SQLite access.

use sqlx::sqlite::SqliteRow;
use sqlx::Row;

#[derive(Debug, Clone)]
pub struct SessionRow {
    pub session_id: String,
    pub role_id: String,
    pub scene_id: String,
    pub created_at: String,
    pub updated_at: String,
    pub message_count: i64,
}

#[derive(Debug, Clone)]
pub struct MessageRow {
    pub id: String,
    pub session_id: String,
    pub turn_index: i32,
    pub sender: String,
    pub content: String,
    pub metadata: Option<String>,
    pub created_at: String,
}

pub(crate) fn session_row_from_tuple(
    (session_id, role_id, scene_id, created_at, updated_at, message_count): (
        String,
        String,
        String,
        String,
        String,
        i64,
    ),
) -> SessionRow {
    SessionRow {
        session_id,
        role_id,
        scene_id,
        created_at,
        updated_at,
        message_count,
    }
}

pub(crate) fn message_row_from_row(row: &SqliteRow) -> MessageRow {
    MessageRow {
        id: row.get("id"),
        session_id: row.get("session_id"),
        turn_index: row.get::<i32, _>("turn_index"),
        sender: row.get("sender"),
        content: row.get("content"),
        metadata: row.get("metadata"),
        created_at: row.get("created_at"),
    }
}

#[derive(Debug, Clone)]
pub struct InsertTurnResult {
    pub message_count: i64,
    pub turn_index: i32,
}

#[derive(Debug, Clone)]
pub struct NewTurnMessages {
    pub user_id: String,
    pub assistant_id: String,
    pub user_content: String,
    pub assistant_content: String,
    pub user_metadata: Option<String>,
    pub assistant_metadata: Option<String>,
    pub user_created_at: String,
    pub assistant_created_at: String,
}

#[derive(Debug, Clone)]
pub struct ChatSearchRow {
    pub id: String,
    pub session_id: String,
    pub turn_index: i32,
    pub sender: String,
    pub content: String,
    pub metadata: Option<String>,
    pub created_at: String,
    pub role_id: String,
    pub scene_id: String,
}

/// Escape SQLite GLOB metacharacters (`*`, `?`, `[`, `]`) in a literal segment.
#[must_use]
pub fn escape_sqlite_glob_literal(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len() + 4);
    for c in raw.chars() {
        match c {
            '*' | '?' | '[' | ']' => {
                out.push('[');
                out.push(c);
                out.push(']');
            }
            other => out.push(other),
        }
    }
    out
}

/// `{manifest_role_id}__sess__*` with GLOB metacharacters in the role id escaped.
#[must_use]
pub fn manifest_sess_glob_pattern(manifest_role_id: &str) -> String {
    format!(
        "{}__sess__*",
        escape_sqlite_glob_literal(manifest_role_id.trim())
    )
}

pub(crate) fn truncate_snippet(s: &str, max_chars: usize) -> String {
    let t = s.trim();
    if t.chars().count() <= max_chars {
        return t.to_string();
    }
    t.chars().take(max_chars).collect::<String>() + "…"
}

#[must_use]
pub fn highlight_snippet(content: &str, query: &str, context_chars: usize) -> String {
    let content_lower = content.to_lowercase();
    let query_lower = query.trim().to_lowercase();
    if query_lower.is_empty() {
        return truncate_snippet(content, context_chars * 2);
    }
    if let Some(pos) = content_lower.find(&query_lower) {
        let start = pos.saturating_sub(context_chars);
        let end = (pos + query_lower.len() + context_chars).min(content.len());
        let slice = &content[start..end];
        let prefix = if start > 0 { "…" } else { "" };
        let suffix = if end < content.len() { "…" } else { "" };
        return format!("{prefix}{slice}{suffix}");
    }
    truncate_snippet(content, context_chars * 2)
}

#[cfg(test)]
mod glob_tests {
    use super::{escape_sqlite_glob_literal, manifest_sess_glob_pattern};

    #[test]
    fn glob_escapes_metacharacters_in_role_id() {
        assert_eq!(escape_sqlite_glob_literal("a*b"), "a[*]b");
        assert_eq!(escape_sqlite_glob_literal("x?y"), "x[?]y");
        assert_eq!(manifest_sess_glob_pattern("r[id*"), "r[[]id[*]__sess__*");
    }
}

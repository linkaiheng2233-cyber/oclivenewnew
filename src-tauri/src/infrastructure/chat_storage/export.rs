//! Export chat sessions / roles as Markdown or JSON (SQLite authoritative).

use super::config::resolve_max_messages_per_session;
use super::db::SessionRow;
use super::mirror;
use crate::error::{AppError, Result};
use crate::infrastructure::db::DbManager;
use chrono::Utc;
use std::path::Path;

use super::types::ChatExportResponse;

fn sanitize_filename(s: &str) -> String {
    s.chars()
        .map(|c| {
            if matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') {
                '_'
            } else {
                c
            }
        })
        .collect()
}

fn format_markdown_session(
    session: &SessionRow,
    role_name: Option<&str>,
    messages: &[super::types::StoredMessage],
) -> String {
    let mut out = String::new();
    let role_label = role_name.unwrap_or(session.role_id.as_str());
    out.push_str(&format!(
        "## Session: {} (role: {}, scene: {})\n\n",
        session.session_id, role_label, session.scene_id
    ));
    out.push_str(&format!(
        "- Created: {}\n- Updated: {}\n- Messages: {}\n\n",
        session.created_at, session.updated_at, session.message_count
    ));
    for msg in messages {
        let sender = if msg.sender == "user" {
            "user"
        } else {
            "assistant"
        };
        out.push_str(&format!(
            "**{sender}** ({timestamp}): {content}\n\n",
            sender = sender,
            timestamp = msg.created_at,
            content = msg.content.replace('\n', "\n  ")
        ));
    }
    out
}

async fn load_session_messages(
    db: &DbManager,
    session_id: &str,
    max_messages: i64,
) -> Result<(SessionRow, Vec<super::types::StoredMessage>)> {
    let session = db
        .get_chat_session(session_id)
        .await?
        .ok_or_else(|| AppError::InvalidParameter(format!("chat session not found: {session_id}")))?;
    let rows = db.fetch_chat_messages(session_id, u32::MAX, 0).await?;
    let messages = rows
        .into_iter()
        .map(|r| super::types::StoredMessage {
            id: r.id,
            session_id: r.session_id,
            turn_index: r.turn_index,
            sender: r.sender,
            content: r.content,
            metadata: r.metadata,
            created_at: r.created_at,
        })
        .collect();
    let _ = max_messages;
    Ok((session, messages))
}

async fn session_json_content(
    db: &DbManager,
    storage_root: &Path,
    session_id: &str,
    max_messages: i64,
) -> Result<String> {
    if storage_root.as_os_str().is_empty() || storage_root == Path::new(".") {
        return session_json_from_db(db, session_id, max_messages).await;
    }
    let path = mirror::rebuild_mirror(db, storage_root, session_id, max_messages).await?;
    let raw = tokio::fs::read_to_string(&path)
        .await
        .map_err(AppError::IoError)?;
    Ok(raw)
}

async fn session_json_from_db(
    db: &DbManager,
    session_id: &str,
    max_messages: i64,
) -> Result<String> {
    let session = db
        .get_chat_session(session_id)
        .await?
        .ok_or_else(|| AppError::InvalidParameter(format!("chat session not found: {session_id}")))?;
    let rows = db.fetch_chat_messages(session_id, u32::MAX, 0).await?;
    let doc = mirror::MirrorDocument::from_session_and_rows(&session, &rows);
    let max = max_messages.max(2) as usize;
    let mut doc = doc;
    if doc.messages.len() > max {
        let drop_n = doc.messages.len() - max;
        doc.messages.drain(0..drop_n);
    }
    serde_json::to_string_pretty(&doc).map_err(|e| AppError::InvalidParameter(e.to_string()))
}

/// Export one session as Markdown or JSON mirror document.
///
/// # Errors
///
/// Propagates DB / IO / validation errors.
pub async fn export_chat_session(
    db: &DbManager,
    storage_root: &Path,
    session_id: &str,
    format: &str,
    max_messages: i64,
    role_name: Option<&str>,
) -> Result<ChatExportResponse> {
    let fmt = format.trim().to_ascii_lowercase();
    let (session, messages) = load_session_messages(db, session_id, max_messages).await?;
    let prefix = sanitize_filename(&session.session_id);
    match fmt.as_str() {
        "markdown" | "md" => {
            let mut body = String::new();
            body.push_str(&format!("# Chat export — {}\n\n", role_name.unwrap_or(&session.role_id)));
            body.push_str(&format!("Exported at: {}\n\n", Utc::now().to_rfc3339()));
            body.push_str(&format_markdown_session(&session, role_name, &messages));
            Ok(ChatExportResponse {
                content: body,
                suggested_filename: format!("{prefix}-chat.md"),
                mime_type: "text/markdown".into(),
                content_encoding: None,
            })
        }
        "json" => {
            let content = session_json_content(db, storage_root, session_id, max_messages).await?;
            Ok(ChatExportResponse {
                content,
                suggested_filename: format!("{prefix}-chat.json"),
                mime_type: "application/json".into(),
                content_encoding: None,
            })
        }
        other => Err(AppError::InvalidParameter(format!(
            "unsupported export format: {other} (use markdown or json)"
        ))),
    }
}

async fn list_manifest_role_sessions(db: &DbManager, role_id: &str) -> Result<Vec<SessionRow>> {
    db.list_chat_sessions_for_manifest_role(role_id).await
}

/// Export all sessions for a manifest role.
///
/// # Errors
///
/// Propagates DB / IO / validation errors.
pub async fn export_role_chats(
    db: &DbManager,
    storage_root: &Path,
    role_id: &str,
    format: &str,
    max_messages: i64,
    role_name: Option<&str>,
) -> Result<ChatExportResponse> {
    let fmt = format.trim().to_ascii_lowercase();
    let sessions = list_manifest_role_sessions(db, role_id).await?;
    if sessions.is_empty() {
        return Err(AppError::InvalidParameter(format!(
            "no chat sessions for role: {role_id}"
        )));
    }
    let rid = sanitize_filename(role_id);
    match fmt.as_str() {
        "markdown" | "md" => {
            let mut body = String::new();
            body.push_str(&format!(
                "# Chat export — {}\n\n",
                role_name.unwrap_or(role_id)
            ));
            body.push_str(&format!("Exported at: {}\n\n", Utc::now().to_rfc3339()));
            for session in &sessions {
                let (_, messages) = load_session_messages(db, &session.session_id, max_messages).await?;
                body.push_str(&format_markdown_session(session, role_name, &messages));
                body.push('\n');
            }
            Ok(ChatExportResponse {
                content: body,
                suggested_filename: format!("{rid}-all-chats.md"),
                mime_type: "text/markdown".into(),
                content_encoding: None,
            })
        }
        "json" => {
            let mut session_docs = Vec::new();
            for session in &sessions {
                let json =
                    session_json_content(db, storage_root, &session.session_id, max_messages)
                        .await?;
                let doc: serde_json::Value = serde_json::from_str(&json)
                    .map_err(|e| AppError::InvalidParameter(e.to_string()))?;
                session_docs.push(doc);
            }
            let combined = serde_json::json!({
                "role_id": role_id,
                "exported_at": Utc::now().to_rfc3339(),
                "sessions": session_docs,
            });
            let content = serde_json::to_string_pretty(&combined)
                .map_err(|e| AppError::InvalidParameter(e.to_string()))?;
            Ok(ChatExportResponse {
                content,
                suggested_filename: format!("{rid}-all-chats.json"),
                mime_type: "application/json".into(),
                content_encoding: None,
            })
        }
        other => Err(AppError::InvalidParameter(format!(
            "unsupported export format: {other} (use markdown or json)"
        ))),
    }
}

#[must_use]
pub fn resolve_export_max_messages(configured: Option<u32>) -> i64 {
    resolve_max_messages_per_session(configured)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::test_db;

    async fn mem_db() -> DbManager {
        test_db::mem_db_manager().await
    }

    #[tokio::test]
    async fn export_session_markdown_contains_sender() {
        let db = mem_db().await;
        let dir = tempfile::tempdir().expect("dir");
        db.upsert_chat_session("mumu", "mumu", "default")
            .await
            .expect("sess");
        db.insert_chat_turn_messages(
            "mumu",
            super::super::db::NewTurnMessages {
                user_id: uuid::Uuid::new_v4().to_string(),
                assistant_id: uuid::Uuid::new_v4().to_string(),
                turn_index: 0,
                user_content: "hello".into(),
                assistant_content: "hi".into(),
                user_metadata: None,
                assistant_metadata: None,
                user_created_at: Utc::now().to_rfc3339(),
                assistant_created_at: Utc::now().to_rfc3339(),
            },
            500,
        )
        .await
        .expect("insert");
        let out = export_chat_session(
            &db,
            dir.path(),
            "mumu",
            "markdown",
            500,
            Some("Mumu"),
        )
        .await
        .expect("export");
        assert!(out.content.contains("**user**"));
        assert!(out.content.contains("hello"));
    }
}

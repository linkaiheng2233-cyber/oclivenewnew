//! Export chat sessions / roles as Markdown or JSON (SQLite authoritative).

use super::config::resolve_max_messages_per_session;
use super::db::SessionRow;
use super::mirror;
use crate::error::{AppError, Result};
use crate::infrastructure::db::DbManager;
use chrono::Utc;
use std::io::{Cursor, Write};
use std::path::Path;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

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
    app_data_dir: &Path,
    session_id: &str,
    max_messages: i64,
) -> Result<String> {
    let path = mirror::rebuild_mirror(db, app_data_dir, session_id, max_messages).await?;
    let raw = tokio::fs::read_to_string(&path)
        .await
        .map_err(AppError::IoError)?;
    Ok(raw)
}

/// Export one session as Markdown or JSON mirror document.
///
/// # Errors
///
/// Propagates DB / IO / validation errors.
pub async fn export_chat_session(
    db: &DbManager,
    app_data_dir: &Path,
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
            let content = session_json_content(db, app_data_dir, session_id, max_messages).await?;
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
    app_data_dir: &Path,
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
            let mut buf = Cursor::new(Vec::new());
            {
                let mut zip = ZipWriter::new(&mut buf);
                let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
                for session in &sessions {
                    let json = session_json_content(db, app_data_dir, &session.session_id, max_messages).await?;
                    let name = format!(
                        "{}/{}.json",
                        sanitize_filename(&session.scene_id),
                        sanitize_filename(&session.session_id)
                    );
                    zip.start_file(name, opts)
                        .map_err(|e| AppError::InvalidParameter(e.to_string()))?;
                    zip.write_all(json.as_bytes())
                        .map_err(|e| AppError::InvalidParameter(e.to_string()))?;
                }
                zip.finish()
                    .map_err(|e| AppError::InvalidParameter(e.to_string()))?;
            }
            use base64::Engine;
            let encoded = base64::engine::general_purpose::STANDARD.encode(buf.into_inner());
            Ok(ChatExportResponse {
                content: encoded,
                suggested_filename: format!("{rid}-all-chats.zip"),
                mime_type: "application/zip".into(),
                content_encoding: Some("base64".into()),
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
    use crate::infrastructure::sqlite_pool;

    async fn mem_db() -> DbManager {
        let pool = sqlite_pool::connect_memory().await.expect("pool");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("migrate");
        DbManager::new(pool)
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

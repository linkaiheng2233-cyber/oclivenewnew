//! JSON file mirror under `{root}/{role_id}/{scene_id}/` (best-effort, DB is authoritative).

use super::config::resolve_session_dir;
use super::db::{MessageRow, SessionRow};
use crate::error::{AppError, Result};
use crate::infrastructure::db::DbManager;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;

pub const MIRROR_SCHEMA_VERSION: i32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirrorMessage {
    pub id: String,
    pub sender: String,
    pub content: String,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn_index: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirrorDocument {
    pub schema_version: i32,
    pub session_id: String,
    pub role_id: String,
    pub scene_id: String,
    pub created_at: String,
    pub updated_at: String,
    pub messages: Vec<MirrorMessage>,
}

impl MirrorDocument {
    pub fn from_session_and_rows(session: &SessionRow, rows: &[MessageRow]) -> Self {
        let messages = rows
            .iter()
            .map(|r| {
                let metadata = r
                    .metadata
                    .as_deref()
                    .and_then(|s| serde_json::from_str(s).ok());
                MirrorMessage {
                    id: r.id.clone(),
                    sender: r.sender.clone(),
                    content: r.content.clone(),
                    timestamp: r.created_at.clone(),
                    turn_index: Some(r.turn_index),
                    metadata,
                }
            })
            .collect();
        Self {
            schema_version: MIRROR_SCHEMA_VERSION,
            session_id: session.session_id.clone(),
            role_id: session.role_id.clone(),
            scene_id: session.scene_id.clone(),
            created_at: session.created_at.clone(),
            updated_at: session.updated_at.clone(),
            messages,
        }
    }
}

/// Filename: `{created_at_compact}_{session_id_prefix}.json`
#[must_use]
pub fn mirror_filename(session: &SessionRow) -> String {
    let prefix: String = session.session_id.chars().take(8).collect();
    let compact = session
        .created_at
        .chars()
        .filter(|c| c.is_ascii_digit())
        .take(15)
        .collect::<String>();
    let compact = if compact.is_empty() {
        "unknown".to_string()
    } else {
        compact
    };
    format!("{compact}_{prefix}.json")
}

pub fn mirror_path_for_session(
    storage_root: &Path,
    session: &SessionRow,
) -> Result<PathBuf> {
    let dir = resolve_session_dir(storage_root, &session.role_id, &session.scene_id)?;
    Ok(dir.join(mirror_filename(session)))
}

/// Append new messages to mirror (or create file). Truncates to [`DEFAULT_MAX_MESSAGES`].
///
/// # Errors
///
/// IO / JSON errors return [`AppError::IoError`] or invalid parameter variants.
pub async fn sync_mirror_append(
    storage_root: &Path,
    session: &SessionRow,
    new_rows: &[MessageRow],
    max_messages: i64,
) -> Result<()> {
    let path = mirror_path_for_session(storage_root, session)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await.map_err(AppError::IoError)?;
    }

    let mut doc = if path.is_file() {
        let raw = fs::read_to_string(&path).await.map_err(AppError::IoError)?;
        serde_json::from_str(&raw).unwrap_or_else(|_| MirrorDocument::from_session_and_rows(session, &[]))
    } else {
        MirrorDocument::from_session_and_rows(session, &[])
    };

    for row in new_rows {
        doc.messages.push(MirrorMessage {
            id: row.id.clone(),
            sender: row.sender.clone(),
            content: row.content.clone(),
            timestamp: row.created_at.clone(),
            turn_index: Some(row.turn_index),
            metadata: row
                .metadata
                .as_deref()
                .and_then(|s| serde_json::from_str(s).ok()),
        });
    }

    let max = max_messages.max(2) as usize;
    if doc.messages.len() > max {
        let drop_n = doc.messages.len() - max;
        doc.messages.drain(0..drop_n);
    }

    doc.updated_at = Utc::now().to_rfc3339();
    write_mirror_atomic(&path, &doc).await
}

/// Rebuild mirror JSON from SQLite (full session messages).
///
/// # Errors
///
/// Database / IO / JSON errors propagate.
pub async fn rebuild_mirror(
    db: &DbManager,
    storage_root: &Path,
    session_id: &str,
    max_messages: i64,
) -> Result<PathBuf> {
    let session = db
        .get_chat_session(session_id)
        .await?
        .ok_or_else(|| AppError::InvalidParameter(format!("chat session not found: {session_id}")))?;
    let rows = db
        .fetch_chat_messages(session_id, u32::MAX, 0)
        .await?;
    let doc = MirrorDocument::from_session_and_rows(&session, &rows);
    let max = max_messages.max(2) as usize;
    let mut doc = doc;
    if doc.messages.len() > max {
        let drop_n = doc.messages.len() - max;
        doc.messages.drain(0..drop_n);
    }
    let path = mirror_path_for_session(storage_root, &session)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await.map_err(AppError::IoError)?;
    }
    write_mirror_atomic(&path, &doc).await?;
    Ok(path)
}

/// Remove mirror JSON for one session (best-effort).
///
/// # Errors
///
/// IO errors propagate.
pub async fn delete_mirror(
    storage_root: &Path,
    role_id: &str,
    scene_id: &str,
    session_id: &str,
) -> Result<()> {
    let dir = resolve_session_dir(storage_root, role_id, scene_id)?;
    if !dir.is_dir() {
        return Ok(());
    }
    let prefix: String = session_id.chars().take(8).collect();
    let mut entries = fs::read_dir(&dir).await.map_err(AppError::IoError)?;
    while let Some(entry) = entries.next_entry().await.map_err(AppError::IoError)? {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.contains(&prefix) && name.ends_with(".json") {
            let _ = fs::remove_file(entry.path()).await;
        }
    }
    Ok(())
}

/// Remove `{root}/{role_id}/` tree (best-effort).
///
/// # Errors
///
/// IO errors propagate as [`AppError::IoError`]; invalid `role_id` segments return [`AppError::InvalidParameter`].
pub async fn delete_mirror_tree_for_role(storage_root: &Path, role_id: &str) -> Result<()> {
    let role_dir = storage_root.join(super::config::sanitize_path_segment(role_id)?);
    if role_dir.is_dir() {
        fs::remove_dir_all(&role_dir).await.map_err(AppError::IoError)?;
    }
    Ok(())
}

async fn write_mirror_atomic(path: &Path, doc: &MirrorDocument) -> Result<()> {
    let json = serde_json::to_string_pretty(doc)
        .map_err(|e| AppError::InvalidParameter(e.to_string()))?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, json).await.map_err(AppError::IoError)?;
    fs::rename(&tmp, path).await.map_err(AppError::IoError)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::db::DbManager;
    use crate::infrastructure::test_db;

    async fn mem_db() -> DbManager {
        test_db::mem_db_manager().await
    }

    #[tokio::test]
    async fn mirror_atomic_write() {
        let dir = tempfile::tempdir().expect("tempdir");
        let app_data = dir.path();
        let storage_root = app_data.join("chats");
        let db = mem_db().await;
        let session = db
            .upsert_chat_session("sess1", "mumu", "default")
            .await
            .expect("sess");
        let row = MessageRow {
            id: "m1".into(),
            session_id: "sess1".into(),
            turn_index: 0,
            sender: "user".into(),
            content: "hello".into(),
            metadata: None,
            created_at: Utc::now().to_rfc3339(),
        };
        sync_mirror_append(
            &storage_root,
            &session,
            std::slice::from_ref(&row),
            crate::infrastructure::chat_storage::config::DEFAULT_MAX_MESSAGES,
        )
            .await
            .expect("sync");
        let path = mirror_path_for_session(&storage_root, &session).expect("path");
        assert!(path.is_file());
        let raw = std::fs::read_to_string(path).expect("read");
        assert!(raw.contains("schema_version"));
        assert!(raw.contains("hello"));
    }
}

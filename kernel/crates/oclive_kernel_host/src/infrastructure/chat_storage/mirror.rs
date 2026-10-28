//! JSON file mirror under `{root}/{role_id}/{scene_id}/` (best-effort, DB is authoritative).

use super::config::load_session_dir;
use super::db::{MessageRow, SessionRow};
use crate::error::{AppError, Result};
use crate::infrastructure::db::DbManager;
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

pub fn mirror_path_for_session(storage_root: &Path, session: &SessionRow) -> Result<PathBuf> {
    let dir = load_session_dir(storage_root, &session.role_id, &session.scene_id)?;
    Ok(dir.join(mirror_filename(session)))
}

fn quarantine_corrupt_mirror(path: &Path) {
    let corrupt_path = path.with_extension("json.corrupt");
    match std::fs::rename(path, &corrupt_path) {
        Ok(()) => tracing::warn!(
            target: "oclive_chat_storage",
            from = %path.display(),
            to = %corrupt_path.display(),
            "quarantined corrupt chat mirror JSON"
        ),
        Err(e) => tracing::warn!(
            target: "oclive_chat_storage",
            path = %path.display(),
            error = %e,
            "failed to quarantine corrupt chat mirror JSON"
        ),
    }
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
    // Read SQLite only after entering the queue, not before waiting behind another writer.
    let guard = super::mirror_io::lock_root(storage_root).await?;
    let session = db.get_chat_session(session_id).await?.ok_or_else(|| {
        AppError::InvalidParameter(format!("chat session not found: {session_id}"))
    })?;
    let rows = db.fetch_chat_messages(session_id, u32::MAX, 0).await?;
    let doc = MirrorDocument::from_session_and_rows(&session, &rows);
    let max = max_messages.max(2) as usize;
    let mut doc = doc;
    if doc.messages.len() > max {
        let drop_n = doc.messages.len() - max;
        doc.messages.drain(0..drop_n);
    }
    let path = mirror_path_for_session(storage_root, &session)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(AppError::IoError)?;
    }
    write_mirror_atomic(&path, &doc, guard).await?;
    Ok(path)
}

/// Remove mirror JSON for one session (exact `session_id` match in file contents).
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
    let dir = load_session_dir(storage_root, role_id, scene_id)?;
    if !storage_root.try_exists()? {
        return Ok(());
    }
    let guard = super::mirror_io::lock_root(storage_root).await?;
    let session_id = session_id.to_owned();
    super::mirror_io::run_locked_io(guard, move || {
        if !dir.is_dir() {
            return Ok(());
        }
        for entry in std::fs::read_dir(&dir)? {
            let path = entry?.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            let raw = std::fs::read_to_string(&path)?;
            match serde_json::from_str::<MirrorDocument>(&raw) {
                Ok(doc) if doc.session_id == session_id => std::fs::remove_file(&path)?,
                Err(error) => {
                    tracing::warn!(target: "oclive_chat_storage", path = %path.display(), %error,
                        "corrupt chat mirror JSON during delete; quarantining");
                    quarantine_corrupt_mirror(&path);
                }
                _ => {}
            }
        }
        Ok(())
    })
    .await
}

/// Remove `{root}/{role_id}/` tree (best-effort).
///
/// # Errors
///
/// IO errors propagate as [`AppError::IoError`]; invalid `role_id` segments return [`AppError::InvalidParameter`].
pub async fn delete_mirror_tree_for_role(storage_root: &Path, role_id: &str) -> Result<()> {
    let role_dir = storage_root.join(super::config::sanitize_path_segment(role_id)?);
    super::mirror_io::delete_tree(storage_root, role_dir).await
}

async fn write_mirror_atomic(
    path: &Path,
    doc: &MirrorDocument,
    guard: tokio::sync::OwnedMutexGuard<()>,
) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(doc)?;
    let path = path.to_owned();
    super::mirror_io::run_locked_io(guard, move || {
        if path.is_file() {
            let previous = std::fs::read(&path)?;
            if serde_json::from_slice::<MirrorDocument>(&previous).is_err() {
                quarantine_corrupt_mirror(&path);
            }
        }
        super::super::file_commit::atomic_write(&path, &bytes)?;
        Ok(())
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::chat_storage::db::NewTurnMessages;
    use crate::infrastructure::db::DbManager;
    use crate::infrastructure::test_db;
    use std::sync::Arc;
    use std::time::Duration;

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
        append_fixture(&db, "sess1", "one").await;
        rebuild_mirror(&db, &storage_root, "sess1", 20)
            .await
            .unwrap();
        let path = mirror_path_for_session(&storage_root, &session).expect("path");
        assert!(path.is_file());
        let raw = std::fs::read_to_string(path).expect("read");
        assert!(raw.contains("schema_version"));
        assert!(raw.contains("one"));
    }

    async fn append_fixture(db: &DbManager, session: &str, id: &str) {
        let timestamp = chrono::Utc::now().to_rfc3339();
        db.insert_chat_turn_messages(
            session,
            NewTurnMessages {
                user_id: format!("{id}-user"),
                assistant_id: format!("{id}-assistant"),
                user_content: id.into(),
                assistant_content: "reply".into(),
                user_metadata: None,
                assistant_metadata: None,
                assistant_emotion_source: None,
                user_created_at: timestamp.clone(),
                assistant_created_at: timestamp,
            },
            100,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn queued_rebuild_reads_latest_edit_and_multiple_writers_keep_all_rows() {
        let root = tempfile::tempdir().unwrap();
        let db = Arc::new(mem_db().await);
        db.upsert_chat_session("queue", "example", "default")
            .await
            .unwrap();
        append_fixture(&db, "queue", "first").await;
        let guard = super::super::mirror_io::lock_root(root.path())
            .await
            .unwrap();
        let mut tasks = tokio::task::JoinSet::new();
        for _ in 0..8 {
            let db = Arc::clone(&db);
            let root = root.path().to_owned();
            tasks.spawn(async move { rebuild_mirror(&db, &root, "queue", 100).await });
        }
        assert!(
            tokio::time::timeout(Duration::from_millis(20), tasks.join_next())
                .await
                .is_err()
        );
        db.edit_chat_message("first-user", "edited after queueing")
            .await
            .unwrap();
        append_fixture(&db, "queue", "second").await;
        drop(guard);
        let mut path = None;
        while let Some(result) = tasks.join_next().await {
            path = Some(result.unwrap().unwrap());
        }
        let raw = std::fs::read(path.unwrap()).unwrap();
        let doc: MirrorDocument = serde_json::from_slice(&raw).unwrap();
        assert_eq!(doc.messages.len(), 4);
        assert!(doc
            .messages
            .iter()
            .any(|row| row.id == "first-user" && row.content == "edited after queueing"));
        assert!(doc.messages.iter().any(|row| row.id == "second-user"));
    }

    #[tokio::test]
    async fn queued_rebuild_does_not_recreate_deleted_session() {
        let root = tempfile::tempdir().unwrap();
        let db = Arc::new(mem_db().await);
        let session = db
            .upsert_chat_session("deleted", "example", "default")
            .await
            .unwrap();
        append_fixture(&db, "deleted", "gone").await;
        let guard = super::super::mirror_io::lock_root(root.path())
            .await
            .unwrap();
        let db_task = Arc::clone(&db);
        let root_task = root.path().to_owned();
        let mut task =
            tokio::spawn(async move { rebuild_mirror(&db_task, &root_task, "deleted", 100).await });
        assert!(tokio::time::timeout(Duration::from_millis(20), &mut task)
            .await
            .is_err());
        db.delete_chat_session("deleted").await.unwrap();
        drop(guard);
        assert!(task.await.unwrap().is_err());
        assert!(!mirror_path_for_session(root.path(), &session)
            .unwrap()
            .exists());
    }
}

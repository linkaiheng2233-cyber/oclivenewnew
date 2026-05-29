//! Auto-cleanup stale chat sessions (SQLite + JSON mirror).

use super::types::AutoCleanupResult;
use crate::error::Result;
use crate::infrastructure::db::DbManager;
use chrono::{DateTime, Duration, Utc};
use std::collections::HashSet;
use std::path::Path;
use tracing::info;

/// Role `chat_storage` config slice used by cleanup.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AutoCleanupConfig {
    pub auto_cleanup_days: Option<u32>,
    pub auto_cleanup_max_sessions: Option<u32>,
    #[serde(default = "default_chat_storage_location_cleanup")]
    pub chat_storage_location: String,
}

fn default_chat_storage_location_cleanup() -> String {
    "global".to_string()
}

impl Default for AutoCleanupConfig {
    fn default() -> Self {
        Self {
            auto_cleanup_days: None,
            auto_cleanup_max_sessions: None,
            chat_storage_location: default_chat_storage_location_cleanup(),
        }
    }
}

impl AutoCleanupConfig {
    #[must_use]
    pub fn from_role_config(cfg: &crate::models::RolePackChatStorageConfig) -> Self {
        Self {
            auto_cleanup_days: cfg.auto_cleanup_days,
            auto_cleanup_max_sessions: cfg.auto_cleanup_max_sessions,
            chat_storage_location: cfg.location.clone(),
        }
    }

    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.auto_cleanup_days.is_some() || self.auto_cleanup_max_sessions.is_some()
    }
}

fn parse_rfc3339(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

fn updated_at_sort_key(s: &str) -> i64 {
    parse_rfc3339(s)
        .map(|dt| dt.timestamp_millis())
        .unwrap_or(0)
}

/// When both policies apply, keep only sessions satisfying **both** (stricter retention).
fn sessions_to_delete(
    sessions: &[(String, String, String, String)],
    cfg: &AutoCleanupConfig,
) -> Vec<String> {
    if !cfg.is_enabled() || sessions.is_empty() {
        return Vec::new();
    }

    let all: HashSet<String> = sessions.iter().map(|(sid, _, _, _)| sid.clone()).collect();
    let mut keep: Option<HashSet<String>> = None;

    if let Some(days) = cfg.auto_cleanup_days {
        let cutoff = Utc::now() - Duration::days(i64::from(days.max(1)));
        let set: HashSet<String> = sessions
            .iter()
            .filter(|(_, _, _, updated_at)| {
                parse_rfc3339(updated_at)
                    .map(|dt| dt >= cutoff)
                    .unwrap_or(true)
            })
            .map(|(sid, _, _, _)| sid.clone())
            .collect();
        keep = Some(match keep {
            None => set,
            Some(k) => k.intersection(&set).cloned().collect(),
        });
    }

    if let Some(max_n) = cfg.auto_cleanup_max_sessions {
        let n = max_n.max(1) as usize;
        let mut sorted = sessions.to_vec();
        sorted.sort_by(|a, b| {
            updated_at_sort_key(&b.3)
                .cmp(&updated_at_sort_key(&a.3))
        });
        let set: HashSet<String> = sorted
            .iter()
            .take(n)
            .map(|(sid, _, _, _)| sid.clone())
            .collect();
        keep = Some(match keep {
            None => set,
            Some(k) => k.intersection(&set).cloned().collect(),
        });
    }

    let keep = keep.unwrap_or(all.clone());
    all.difference(&keep).cloned().collect()
}

/// Apply auto-cleanup for one manifest role. Best-effort mirror removal per session.
///
/// # Errors
///
/// Database errors propagate; mirror IO failures are logged and skipped.
pub async fn apply_auto_cleanup(
    db: &DbManager,
    storage_root: &Path,
    role_id: &str,
    cfg: &AutoCleanupConfig,
) -> Result<AutoCleanupResult> {
    if !cfg.is_enabled() {
        return Ok(AutoCleanupResult::default());
    }

    let rows = db.list_chat_sessions_for_manifest_role(role_id).await?;
    let indexed: Vec<(String, String, String, String)> = rows
        .iter()
        .map(|r| {
            (
                r.session_id.clone(),
                r.role_id.clone(),
                r.scene_id.clone(),
                r.updated_at.clone(),
            )
        })
        .collect();

    let to_delete = sessions_to_delete(&indexed, cfg);
    if to_delete.is_empty() {
        return Ok(AutoCleanupResult::default());
    }

    let mut bytes_freed = 0u64;
    for sid in &to_delete {
        if let Some(row) = rows.iter().find(|r| r.session_id == *sid) {
            if let Ok(n) = super::stats::mirror_file_bytes_for_session(storage_root, row).await {
                bytes_freed = bytes_freed.saturating_add(n);
            }
            let _ = super::mirror::delete_mirror(
                storage_root,
                &row.role_id,
                &row.scene_id,
                sid,
            )
            .await;
        }
        db.delete_chat_session(sid).await?;
    }

    info!(
        target: "oclive_chat_storage",
        role_id = %role_id,
        sessions_deleted = to_delete.len(),
        bytes_freed,
        "auto_cleanup completed"
    );

    Ok(AutoCleanupResult {
        sessions_deleted: to_delete.len() as u32,
        bytes_freed,
    })
}

/// SQLite-only cleanup (no mirror files).
/// Apply auto-cleanup for one role using SQLite only (no mirror deletes).
///
/// # Errors
///
/// Propagates database errors.
pub async fn apply_auto_cleanup_sqlite(
    db: &DbManager,
    role_id: &str,
    cfg: &AutoCleanupConfig,
) -> Result<AutoCleanupResult> {
    if !cfg.is_enabled() {
        return Ok(AutoCleanupResult::default());
    }
    let rows = db.list_chat_sessions_for_manifest_role(role_id).await?;
    let indexed: Vec<(String, String, String, String)> = rows
        .iter()
        .map(|r| {
            (
                r.session_id.clone(),
                r.role_id.clone(),
                r.scene_id.clone(),
                r.updated_at.clone(),
            )
        })
        .collect();
    let to_delete = sessions_to_delete(&indexed, cfg);
    for sid in &to_delete {
        db.delete_chat_session(sid).await?;
    }
    Ok(AutoCleanupResult {
        sessions_deleted: to_delete.len() as u32,
        bytes_freed: 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_sessions_deletes_oldest() {
        let sessions = vec![
            ("a".into(), "r".into(), "d".into(), "2026-05-03T00:00:00Z".into()),
            ("b".into(), "r".into(), "d".into(), "2026-05-02T00:00:00Z".into()),
            ("c".into(), "r".into(), "d".into(), "2026-05-01T00:00:00Z".into()),
        ];
        let cfg = AutoCleanupConfig {
            auto_cleanup_days: None,
            auto_cleanup_max_sessions: Some(2),
            chat_storage_location: "global".into(),
        };
        let del = sessions_to_delete(&sessions, &cfg);
        assert_eq!(del, vec!["c".to_string()]);
    }
}

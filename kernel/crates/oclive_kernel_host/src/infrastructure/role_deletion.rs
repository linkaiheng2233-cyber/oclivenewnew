//! File quarantine plus a commit marker in the same SQLite transaction as role deletion.
//! Prepared records restore files after interruption; committed records only finish cleanup.

use super::db::DbManager;
use super::directory_plugins::DirectoryPluginRuntime;
use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct PendingRoleDeletion {
    role_id: String,
    operation_id: uuid::Uuid,
    roles_root: PathBuf,
    originals: Vec<PathBuf>,
    committed: bool,
}

impl PendingRoleDeletion {
    fn key(&self) -> String {
        format!("role_delete:{}", self.operation_id)
    }

    fn staged(&self, original: &Path) -> Result<PathBuf> {
        let parent = original
            .parent()
            .ok_or_else(|| AppError::InvalidParameter("role deletion path has no parent".into()))?;
        Ok(parent.join(format!(
            ".oclive-delete-{}-{}",
            self.operation_id, self.role_id
        )))
    }

    fn validate(&self, root: &Path) -> Result<()> {
        oclive_validation::validate_role_id(&self.role_id).map_err(AppError::InvalidParameter)?;
        if self.operation_id.is_nil() || self.roles_root != root.canonicalize()? {
            return Err(AppError::InvalidParameter(
                "role deletion recovery root mismatch".into(),
            ));
        }
        for original in &self.originals {
            if original.file_name().and_then(|s| s.to_str()) != Some(self.role_id.as_str()) {
                return Err(AppError::InvalidParameter(
                    "invalid role deletion recovery path".into(),
                ));
            }
            let parent = original
                .parent()
                .ok_or_else(|| AppError::InvalidParameter("missing role deletion parent".into()))?;
            if parent.canonicalize()? != parent {
                return Err(AppError::InvalidParameter(
                    "role deletion parent changed".into(),
                ));
            }
            for path in [original.clone(), self.staged(original)?] {
                match fs::symlink_metadata(&path) {
                    Ok(metadata)
                        if !metadata.is_dir()
                            || metadata.file_type().is_symlink()
                            || path.canonicalize()? != path =>
                    {
                        return Err(AppError::InvalidParameter(
                            "linked/invalid role deletion directory".into(),
                        ));
                    }
                    Ok(_) => {}
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(error) => return Err(error.into()),
                }
            }
        }
        Ok(())
    }

    fn prepare(root: &Path, role_id: &str, mirror: &Path) -> Result<Self> {
        oclive_validation::validate_role_id(role_id).map_err(AppError::InvalidParameter)?;
        let root = root.canonicalize()?;
        let role = root.join(role_id);
        let mut originals = Vec::new();
        if role.try_exists()? {
            originals.push(role.clone());
        }
        if mirror.try_exists()? {
            let parent = mirror
                .parent()
                .ok_or_else(|| AppError::InvalidParameter("mirror path has no parent".into()))?
                .canonicalize()?;
            let mirror = parent.join(role_id);
            if !mirror.starts_with(&role) {
                originals.push(mirror);
            }
        }
        let record = Self {
            role_id: role_id.into(),
            operation_id: uuid::Uuid::new_v4(),
            roles_root: root.clone(),
            originals,
            committed: false,
        };
        record.validate(&root)?;
        Ok(record)
    }

    fn stage(&self) -> Result<()> {
        self.validate(&self.roles_root)?;
        for original in &self.originals {
            fs::rename(original, self.staged(original)?)?;
        }
        Ok(())
    }

    fn restore(&self) -> Result<()> {
        self.validate(&self.roles_root)?;
        for original in self.originals.iter().rev() {
            let staged = self.staged(original)?;
            if staged.try_exists()? {
                if original.try_exists()? {
                    return Err(AppError::InvalidParameter(format!(
                        "role recovery conflict; backup retained at {}",
                        staged.display()
                    )));
                }
                fs::rename(staged, original)?;
            }
        }
        Ok(())
    }

    fn cleanup(&self) -> Result<()> {
        self.validate(&self.roles_root)?;
        for original in &self.originals {
            let staged = self.staged(original)?;
            if staged.try_exists()? {
                fs::remove_dir_all(staged)?;
            }
        }
        Ok(())
    }
}

/// Caller holds the roles write lock; no permanent deletion happens before the DB commit.
pub(crate) async fn delete_files_and_data(
    db: &DbManager,
    root: &Path,
    role_id: &str,
    mirror: &Path,
) -> Result<(PendingRoleDeletion, Vec<String>)> {
    let mirror_root = mirror
        .parent()
        .ok_or_else(|| AppError::InvalidParameter("role mirror directory has no parent".into()))?;
    // Finish any already-started mirror publication before quarantining files and committing.
    let _mirror_guard = super::chat_storage::lock_mirror_root(mirror_root).await?;
    let mut record = PendingRoleDeletion::prepare(root, role_id, mirror)?;
    let key = record.key();
    db.upsert_app_setting(&key, &serde_json::to_string(&record)?)
        .await?;
    if let Err(error) = record.stage() {
        record.restore()?;
        db.clear_role_deletion(&key).await?;
        return Err(error);
    }
    record.committed = true;
    let committed = serde_json::to_string(&record)?;
    match db
        .delete_role_data_and_mark_cleanup(role_id, Some((&key, &committed)))
        .await
    {
        Ok(ids) => Ok((record, ids)),
        Err(error) => {
            // A failed/ambiguous commit is resolved from the durable marker, never guessed.
            let saved = db.get_app_setting(&key).await?.ok_or_else(|| {
                AppError::DatabaseError("role deletion recovery record missing".into())
            })?;
            let saved: PendingRoleDeletion = serde_json::from_str(&saved)?;
            if saved.committed {
                return Ok((saved, Vec::new()));
            }
            saved.restore()?;
            db.clear_role_deletion(&key).await?;
            Err(error)
        }
    }
}

/// Post-commit failures keep the record for restart/retry rather than reporting deletion rolled back.
pub(crate) async fn finish_deletion(
    db: &DbManager,
    record: &PendingRoleDeletion,
    plugins: &DirectoryPluginRuntime,
) -> Result<()> {
    // A newly imported role with the same id owns its new UI state.
    if !record.roles_root.join(&record.role_id).try_exists()? {
        plugins
            .remove_role_plugin_state(&record.role_id)
            .map_err(AppError::InvalidParameter)?;
    }
    let owned = record.clone();
    tokio::task::spawn_blocking(move || owned.cleanup())
        .await
        .map_err(|error| AppError::InvalidParameter(format!("role cleanup task: {error}")))??;
    db.clear_role_deletion(&record.key()).await
}

pub(crate) async fn recover_pending_deletions(
    db: &DbManager,
    root: &Path,
    plugins: Option<&DirectoryPluginRuntime>,
) -> Result<()> {
    let records = db.pending_role_deletions().await?;
    if records.is_empty() {
        return Ok(());
    }
    let _guard = super::role_files::lock_role_writes(root)?;
    for (key, raw) in records {
        let record: PendingRoleDeletion = serde_json::from_str(&raw)?;
        if record.key() != key {
            return Err(AppError::InvalidParameter(
                "invalid role deletion key".into(),
            ));
        }
        record.validate(root)?;
        if record.committed {
            if let Some(plugins) = plugins {
                if let Err(error) = finish_deletion(db, &record, plugins).await {
                    tracing::warn!(target: "oclive_role_files", %error, role_id = %record.role_id,
                        "role deleted; deferred cleanup remains pending");
                }
            }
        } else {
            record.restore()?;
            db.clear_role_deletion(&key).await?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::test_db::mem_db_manager;

    #[tokio::test]
    async fn deletion_waits_for_active_mirror_before_staging_and_committing() {
        let root = tempfile::tempdir().unwrap();
        let roles = root.path().join("fixture-role-library");
        let mirrors = root.path().join("chats");
        fs::create_dir_all(roles.join("example")).unwrap();
        fs::create_dir_all(mirrors.join("example")).unwrap();
        let db = std::sync::Arc::new(mem_db_manager().await);
        db.upsert_chat_session("example", "example", "default")
            .await
            .unwrap();
        let guard = super::super::chat_storage::lock_mirror_root(&mirrors)
            .await
            .unwrap();
        let db_task = std::sync::Arc::clone(&db);
        let roles_task = roles.clone();
        let mirror_task = mirrors.join("example");
        let mut task = tokio::spawn(async move {
            delete_files_and_data(&db_task, &roles_task, "example", &mirror_task).await
        });
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(20), &mut task)
                .await
                .is_err()
        );
        assert!(roles.join("example").is_dir());
        assert!(db.get_chat_session("example").await.unwrap().is_some());
        drop(guard);
        let (record, _) = task.await.unwrap().unwrap();
        assert!(record.committed);
        assert!(!roles.join("example").exists());
        assert!(db.get_chat_session("example").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn interrupted_precommit_restores_files_on_restart() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join("example")).unwrap();
        fs::write(root.path().join("example/saved.txt"), "saved").unwrap();
        let db = mem_db_manager().await;
        db.ensure_role_runtime("example").await.unwrap();
        let record =
            PendingRoleDeletion::prepare(root.path(), "example", &root.path().join("missing"))
                .unwrap();
        db.upsert_app_setting(&record.key(), &serde_json::to_string(&record).unwrap())
            .await
            .unwrap();
        record.stage().unwrap();
        assert!(!root.path().join("example").exists());
        recover_pending_deletions(&db, root.path(), None)
            .await
            .unwrap();
        assert_eq!(
            fs::read(root.path().join("example/saved.txt")).unwrap(),
            b"saved"
        );
        assert!(db.role_runtime_exists("example").await.unwrap());
        assert!(db.pending_role_deletions().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn database_failure_restores_files_and_data() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join("example")).unwrap();
        fs::write(root.path().join("example/saved.txt"), "saved").unwrap();
        let db = mem_db_manager().await;
        db.ensure_role_runtime("example").await.unwrap();
        sqlx::query("CREATE TRIGGER block_review_delete BEFORE DELETE ON role_runtime BEGIN SELECT RAISE(ABORT, 'fixture failure'); END")
            .execute(&db.pool).await.unwrap();
        assert!(
            delete_files_and_data(&db, root.path(), "example", &root.path().join("missing"))
                .await
                .is_err()
        );
        assert!(root.path().join("example/saved.txt").is_file());
        assert!(db.role_runtime_exists("example").await.unwrap());
        assert!(db.pending_role_deletions().await.unwrap().is_empty());
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn locked_role_directory_does_not_delete_database() {
        use std::os::windows::fs::OpenOptionsExt;
        let root = tempfile::tempdir().unwrap();
        let role = root.path().join("example");
        fs::create_dir(&role).unwrap();
        fs::write(role.join("saved.txt"), "saved").unwrap();
        let db = mem_db_manager().await;
        db.ensure_role_runtime("example").await.unwrap();
        // Real Windows sharing denial; only the synthetic role directory is held open.
        let held = fs::OpenOptions::new()
            .read(true)
            .share_mode(0)
            .custom_flags(0x02000000)
            .open(&role)
            .unwrap();
        let result =
            delete_files_and_data(&db, root.path(), "example", &root.path().join("missing")).await;
        drop(held);
        assert!(result.is_err());
        assert_eq!(fs::read(role.join("saved.txt")).unwrap(), b"saved");
        assert!(db.role_runtime_exists("example").await.unwrap());
        assert!(db.pending_role_deletions().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn committed_deletion_is_not_restored_after_restart() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join("example")).unwrap();
        let db = mem_db_manager().await;
        db.ensure_role_runtime("example").await.unwrap();
        let (record, _) =
            delete_files_and_data(&db, root.path(), "example", &root.path().join("missing"))
                .await
                .unwrap();
        recover_pending_deletions(&db, root.path(), None)
            .await
            .unwrap();
        assert!(!root.path().join("example").exists());
        assert!(!db.role_runtime_exists("example").await.unwrap());
        record.cleanup().unwrap();
        record.cleanup().unwrap();
        assert!(record.committed);
    }
}

//! Serialized, recoverable replacement of a role directory. No role contents are deleted
//! until the replacement is completely copied and validated by the caller.

use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::path::Path;

const IMPORT_PREFIX: &str = ".oclive-role-import-";

pub(crate) fn lock_role_writes(root: &Path) -> Result<File> {
    fs::create_dir_all(root)?;
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(root.join(".oclive-role-write.lock"))?;
    file.try_lock().map_err(|error| {
        AppError::InvalidParameter(format!(
            "role files are busy; retry after the other import/delete finishes: {error}"
        ))
    })?;
    Ok(file)
}

#[derive(Serialize, Deserialize)]
struct ImportRecord {
    role_id: String,
}

pub(crate) fn import_workspace(root: &Path) -> Result<tempfile::TempDir> {
    Ok(tempfile::Builder::new()
        .prefix(IMPORT_PREFIX)
        .tempdir_in(root)?)
}

pub(crate) fn commit_import(workspace: tempfile::TempDir, dest: &Path, id: &str) -> Result<()> {
    let record = serde_json::to_vec(&ImportRecord {
        role_id: id.to_string(),
    })?;
    super::file_commit::atomic_write(&workspace.path().join("import.json"), &record)?;
    // A crash must leave the previous role on disk, rather than a TempDir dropping it.
    let workspace = workspace.keep();
    let previous = workspace.join("previous");
    if dest.exists() {
        fs::rename(dest, &previous)?;
    }
    if let Err(error) = fs::rename(workspace.join("next").join(id), dest) {
        if previous.exists() {
            fs::rename(&previous, dest).map_err(|rollback| AppError::InvalidParameter(format!(
                "role replacement failed ({error}); restore failed ({rollback}); backup retained at {}",
                previous.display()
            )))?;
        }
        return Err(error.into());
    }
    cleanup_workspace(&workspace);
    Ok(())
}

fn cleanup_workspace(path: &Path) {
    if let Err(error) = fs::remove_dir_all(path) {
        tracing::warn!(target: "oclive_role_files", path = %path.display(), %error,
            "role import completed; replacement backup cleanup will retry at startup");
    }
}

/// Caller holds the role write lock. Only journals created by commit_import are replayed.
pub(crate) fn recover_imports_locked(root: &Path) -> Result<()> {
    let root = root.canonicalize()?;
    for entry in fs::read_dir(&root)? {
        let entry = entry?;
        let name = entry.file_name();
        if !name.to_string_lossy().starts_with(IMPORT_PREFIX) {
            continue;
        }
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let workspace = entry.path().canonicalize()?;
        if workspace.parent() != Some(root.as_path()) {
            return Err(AppError::InvalidParameter(
                "import workspace escapes roles root".into(),
            ));
        }
        let record_path = workspace.join("import.json");
        if !record_path.is_file() {
            continue;
        } // Interrupted copy has not touched a role.
        let record: ImportRecord = serde_json::from_slice(&fs::read(record_path)?)?;
        oclive_validation::validate_role_id(&record.role_id).map_err(AppError::InvalidParameter)?;
        let dest = root.join(&record.role_id);
        let previous = workspace.join("previous");
        let next = workspace.join("next").join(&record.role_id);
        if previous.exists() && next.exists() && dest.exists() {
            return Err(AppError::InvalidParameter(format!(
                "role import recovery conflict; role and backup retained at {}",
                workspace.display()
            )));
        }
        if previous.exists() && !dest.exists() {
            fs::rename(&previous, &dest)?;
        }
        cleanup_workspace(&workspace);
    }
    Ok(())
}

pub(crate) fn recover_imports(root: &Path) -> Result<()> {
    if !root.is_dir() {
        return Ok(());
    }
    // Ordinary read-only bundles do not need a write lock when no import was interrupted.
    let mut pending = false;
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if entry
            .file_name()
            .to_string_lossy()
            .starts_with(IMPORT_PREFIX)
            && entry.path().join("import.json").try_exists()?
        {
            pending = true;
            break;
        }
    }
    if pending {
        let _guard = lock_role_writes(root)?;
        recover_imports_locked(root)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interrupted_swap_restores_previous_role() {
        let root = tempfile::tempdir().unwrap();
        let workspace = import_workspace(root.path()).unwrap().keep();
        fs::create_dir(workspace.join("previous")).unwrap();
        fs::write(workspace.join("previous/old.txt"), "saved").unwrap();
        fs::create_dir_all(workspace.join("next/example")).unwrap();
        fs::write(workspace.join("import.json"), br#"{"role_id":"example"}"#).unwrap();
        recover_imports(root.path()).unwrap();
        assert_eq!(
            fs::read(root.path().join("example/old.txt")).unwrap(),
            b"saved"
        );
    }

    #[test]
    fn failed_install_restores_existing_role() {
        let root = tempfile::tempdir().unwrap();
        let dest = root.path().join("example");
        fs::create_dir(&dest).unwrap();
        fs::write(dest.join("old.txt"), "saved").unwrap();
        let workspace = import_workspace(root.path()).unwrap();
        // No `next` directory: deterministic rename failure after backing up the old role.
        assert!(commit_import(workspace, &dest, "example").is_err());
        assert_eq!(fs::read(dest.join("old.txt")).unwrap(), b"saved");
        recover_imports(root.path()).unwrap();
        assert!(dest.join("old.txt").is_file());
    }
}

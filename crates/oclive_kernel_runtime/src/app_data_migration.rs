//! One-time copy of legacy Tauri app data into canonical `OCLive/data`.

use crate::paths::{self, ENV_SKIP_APP_DATA_MIGRATION, TAURI_APP_IDENTIFIER};
use std::fs;
use std::path::Path;

const MIGRATION_MARKER: &str = ".migrated_from_tauri";

/// Run before opening `app.db` on canonical host paths.
///
/// # Errors
///
/// Returns an error when migration is required but copying fails (caller must not open DB for write).
pub fn ensure_canonical_app_data_ready(canonical: &Path) -> Result<(), String> {
    if paths::env_flag_is_truthy(ENV_SKIP_APP_DATA_MIGRATION) {
        let _ = fs::create_dir_all(canonical);
        return Ok(());
    }

    fs::create_dir_all(canonical).map_err(|e| format!("create canonical dir: {e}"))?;
    let db = paths::resolve_db_path(canonical);
    if db.is_file() {
        return Ok(());
    }

    let legacy = paths::tauri_legacy_app_data_dir();
    let legacy_db = paths::resolve_db_path(&legacy);
    if !legacy_db.is_file() {
        return Ok(());
    }

    copy_dir_recursive(&legacy, canonical)?;
    write_migration_marker(canonical, &legacy)?;
    tracing::info!(
        target: "oclive_app_data",
        from = %legacy.display(),
        to = %canonical.display(),
        "migrated app data from Tauri legacy path"
    );
    Ok(())
}

fn write_migration_marker(canonical: &Path, legacy: &Path) -> Result<(), String> {
    let marker = canonical.join(MIGRATION_MARKER);
    let body = format!(
        "source={}\nmigrated_at={}\nidentifier={}\n",
        legacy.display(),
        chrono::Utc::now().to_rfc3339(),
        TAURI_APP_IDENTIFIER
    );
    fs::write(&marker, body).map_err(|e| format!("write migration marker: {e}"))
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    if !src.is_dir() {
        return Err(format!(
            "legacy app_data is not a directory: {}",
            src.display()
        ));
    }
    for entry in fs::read_dir(src).map_err(|e| format!("read legacy dir: {e}"))? {
        let entry = entry.map_err(|e| format!("read legacy entry: {e}"))?;
        let name = entry.file_name();
        let from = entry.path();
        let to = dst.join(&name);
        if from.is_dir() {
            fs::create_dir_all(&to).map_err(|e| format!("mkdir {}: {e}", to.display()))?;
            copy_dir_recursive(&from, &to)?;
        } else {
            if let Some(parent) = to.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("mkdir parent {}: {e}", parent.display()))?;
            }
            fs::copy(&from, &to)
                .map_err(|e| format!("copy {} -> {}: {e}", from.display(), to.display()))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migration_copies_legacy_tree() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let legacy = tmp.path().join("legacy");
        let canonical = tmp.path().join("canonical");
        fs::create_dir_all(&legacy).unwrap();
        fs::write(legacy.join("app.db"), b"sqlite").unwrap();
        fs::write(legacy.join("plugin_state.json"), b"{}").unwrap();
        copy_dir_recursive(&legacy, &canonical).unwrap();
        assert!(canonical.join("app.db").is_file());
        assert!(canonical.join("plugin_state.json").is_file());
    }
}

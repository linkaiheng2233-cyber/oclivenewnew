//! Protect persisted chat data before the canonical directory is initialized.

use std::io::ErrorKind;
use std::path::Path;

/// Check that the history database can be initialized without hiding local chats.
///
/// # Errors
///
/// Returns an error on a filesystem inspection failure, or if local chat
/// mirrors remain while the authoritative database is missing or zero bytes.
pub fn check_persistent_history_before_init(canonical: &Path) -> Result<(), String> {
    let db = oclive_kernel_runtime::find_db_path(canonical);
    let needs_database = match std::fs::metadata(&db) {
        Ok(metadata) => metadata.is_file() && metadata.len() == 0,
        Err(error) if error.kind() == ErrorKind::NotFound => true,
        Err(error) => return Err(format!("Inspect database {}: {error}", db.display())),
    };
    if needs_database && super::chat_storage::has_local_chat_mirror(canonical)? {
        let error = format!(
            "The database {} is missing or empty, and local chat mirrors are still present. Restore a verified database backup for this data directory before starting the kernel.",
            db.display()
        );
        tracing::error!(target: "oclive_app_data", %error, "persistent database initialization blocked");
        return Err(error);
    }
    Ok(())
}

/// Prepare the persistent data directory for the desktop or HTTP kernel.
///
/// # Errors
///
/// Returns a history preflight, filesystem, or migration error.
pub fn ensure_canonical_app_data_ready(canonical: &Path) -> Result<(), String> {
    check_persistent_history_before_init(canonical)?;
    oclive_kernel_runtime::ensure_canonical_app_data_ready(canonical)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn local_chat_mirrors_prevent_missing_or_empty_database_initialization() {
        let temp = tempfile::tempdir().expect("isolated app data");
        let scene = temp.path().join("chats/mumu/home");
        fs::create_dir_all(&scene).expect("mirror directory");
        let mirror = scene.join("202611040433179_fixture.json");
        let contents = br#"{"schema_version":1,"session_id":"fixture","messages":[{"sender":"user","content":"fixture"}]}"#;
        fs::write(&mirror, contents).expect("fixture mirror");
        let db = oclive_kernel_runtime::find_db_path(temp.path());

        let missing = ensure_canonical_app_data_ready(temp.path()).expect_err("missing DB guard");
        assert!(missing.contains("local chat mirrors are still present"));
        assert!(!db.exists());

        fs::write(&db, []).expect("zero-byte DB placeholder");
        let empty = ensure_canonical_app_data_ready(temp.path()).expect_err("empty DB guard");
        assert_eq!(empty, missing);
        assert_eq!(fs::metadata(&db).expect("placeholder retained").len(), 0);
        assert_eq!(fs::read(&mirror).expect("mirror retained"), contents);
    }
}

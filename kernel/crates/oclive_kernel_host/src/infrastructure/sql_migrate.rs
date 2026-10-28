//! Runtime SQLite migrations without the `sqlx` umbrella `migrate` feature (avoids mysql/postgres in the lockfile).

use oclive_kernel_runtime::{find_monorepo_root, ENV_ROLES_DIR};
use sha2::{Digest, Sha256};
use sqlx::sqlite::SqlitePool;
use sqlx::Executor;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Override SQLite migrations directory (must contain `*.sql`).
pub const ENV_MIGRATIONS_DIR: &str = "OCLIVE_MIGRATIONS_DIR";

const HOST_MIGRATIONS_REL: &str = "kernel/crates/oclive_kernel_host/migrations";
const LEGACY_HOST_MIGRATIONS_REL: &str = "crates/oclive_kernel_host/migrations";
const LEGACY_MIGRATIONS_REL: &str = "distros/desktop-tauri/migrations";

/// Returns true when `path` is a directory containing at least one `*.sql` file.
#[must_use]
pub fn is_migrations_dir(path: &Path) -> bool {
    if !path.is_dir() {
        return false;
    }
    std::fs::read_dir(path)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .any(|entry| entry.path().extension().is_some_and(|ext| ext == "sql"))
}

fn migration_discovery_anchors() -> Vec<PathBuf> {
    let mut anchors = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            anchors.push(parent.to_path_buf());
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        anchors.push(cwd);
    }
    for key in ["OCLIVE_LOCAL_MONOREPO", ENV_ROLES_DIR] {
        if let Ok(raw) = std::env::var(key) {
            let trimmed = raw.trim();
            if !trimmed.is_empty() {
                anchors.push(PathBuf::from(trimmed));
            }
        }
    }
    anchors
}

fn bundled_migration_candidates(anchors: &[PathBuf]) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    for anchor in anchors {
        let mut nearby = vec![
            anchor.join("migrations"),
            anchor.join("resources").join("migrations"),
        ];
        if anchor.file_name().is_some_and(|name| name == "roles") {
            let Some(parent) = anchor.parent() else {
                continue;
            };
            nearby.push(parent.join("migrations"));
        }
        for candidate in nearby {
            if !candidates.contains(&candidate) {
                candidates.push(candidate);
            }
        }
    }
    candidates
}

fn local_migration_candidates(anchors: &[PathBuf], build_tree: PathBuf) -> Vec<PathBuf> {
    let mut candidates = bundled_migration_candidates(anchors);
    // A release must use SQL shipped with its runner. The build machine may
    // still have a checkout whose migrations have advanced since this build.
    candidates.push(build_tree);
    candidates
}

/// Resolve migrations directory for runtime apply.
///
/// Order: `OCLIVE_MIGRATIONS_DIR` → executable/cwd-relative bundle →
/// compile-time crate directory → monorepo
/// `kernel/crates/oclive_kernel_host/migrations` (via `find_monorepo_root`) → legacy
/// `crates/oclive_kernel_host/migrations` → `src-tauri/migrations`.
///
/// # Errors
///
/// Returns a message listing attempted paths when none contain migration SQL.
pub fn find_migrations_dir() -> Result<PathBuf, String> {
    let mut tried: Vec<String> = Vec::new();

    if let Ok(raw) = std::env::var(ENV_MIGRATIONS_DIR) {
        let path = PathBuf::from(raw.trim());
        if is_migrations_dir(&path) {
            tracing::info!(
                target: "oclive_migrate",
                dir = %path.display(),
                "using migrations from OCLIVE_MIGRATIONS_DIR"
            );
            return Ok(path);
        }
        tried.push(format!(
            "{ENV_MIGRATIONS_DIR}={} (missing or no .sql)",
            path.display()
        ));
    }

    let anchors = migration_discovery_anchors();
    let build_tree = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("migrations");
    for candidate in local_migration_candidates(&anchors, build_tree) {
        if is_migrations_dir(&candidate) {
            tracing::info!(
                target: "oclive_migrate",
                dir = %candidate.display(),
                "using local migrations"
            );
            return Ok(candidate);
        }
        tried.push(format!("{} (missing or no .sql)", candidate.display()));
    }

    if let Some(repo) = find_monorepo_root(&anchors) {
        for rel in [
            HOST_MIGRATIONS_REL,
            LEGACY_HOST_MIGRATIONS_REL,
            LEGACY_MIGRATIONS_REL,
        ] {
            let candidate = repo.join(rel);
            if is_migrations_dir(&candidate) {
                tracing::info!(
                    target: "oclive_migrate",
                    dir = %candidate.display(),
                    "using migrations from monorepo discovery"
                );
                return Ok(candidate);
            }
            tried.push(format!("{} (missing or no .sql)", candidate.display()));
        }
    } else {
        tried.push(format!(
            "monorepo root not found from {} anchor(s)",
            anchors.len()
        ));
    }

    Err(format!(
        "no SQLite migrations directory found; tried: {}",
        tried.join("; ")
    ))
}

/// Create a transactionally consistent SQLite snapshot before migrations (file DB only).
///
/// `std::fs::copy` is not safe for a database in WAL mode because committed pages may still
/// live only in `app.db-wal`. `VACUUM INTO` reads through SQLite and writes a standalone,
/// integrity-checkable database that includes the committed WAL state.
///
/// # Errors
///
/// Returns an error when the snapshot cannot be created.
pub async fn backup_db_file(
    db: &SqlitePool,
    db_file: &Path,
    app_data_dir: &Path,
) -> Result<PathBuf, String> {
    if !db_file.is_file() {
        return Ok(PathBuf::new());
    }
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let dest = app_data_dir.join(format!("app.db.bak.{ts}"));
    std::fs::create_dir_all(app_data_dir)
        .map_err(|e| format!("backup mkdir {}: {e}", app_data_dir.display()))?;
    let dest_text = dest.to_string_lossy().into_owned();
    if let Err(error) = sqlx::query("VACUUM INTO ?")
        .bind(dest_text)
        .execute(db)
        .await
    {
        let _ = std::fs::remove_file(&dest);
        return Err(format!(
            "backup {} -> {}: {error}",
            db_file.display(),
            dest.display()
        ));
    }
    tracing::info!(
        target: "oclive_migrate",
        from = %db_file.display(),
        to = %dest.display(),
        "database backup before migration"
    );
    Ok(dest)
}

/// Restore `db_file` from a backup path after a failed migration attempt.
///
/// # Errors
///
/// Returns an error when restore copy fails.
pub fn restore_db_from_backup(db_file: &Path, backup: &Path) -> Result<(), String> {
    let metadata = retry_windows_sharing_violation(|| std::fs::metadata(backup))
        .map_err(|error| format!("read backup {}: {error}", backup.display()))?;
    if !metadata.is_file() {
        return Err(format!("backup is not a file: {}", backup.display()));
    }
    if let Some(parent) = db_file.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("restore mkdir {}: {e}", parent.display()))?;
    }
    // The caller must close every pool first. A WAL belongs to the database image it was
    // created from; retaining it while replacing the main file can replay incompatible pages.
    for suffix in ["wal", "shm"] {
        let sidecar = PathBuf::from(format!("{}-{suffix}", db_file.to_string_lossy()));
        if sidecar.is_file() {
            retry_windows_sharing_violation(|| std::fs::remove_file(&sidecar))
                .map_err(|e| format!("remove stale SQLite sidecar {}: {e}", sidecar.display()))?;
        }
    }
    retry_windows_sharing_violation(|| std::fs::copy(backup, db_file))
        .map_err(|e| format!("restore {} <- {}: {e}", db_file.display(), backup.display()))?;
    Ok(())
}

// A closed SQLx pool may have a SQLite worker finishing file-handle teardown on Windows.
// Retry only the observed sharing-violation code, with a 200 ms budget per operation.
// ACL, missing-file and disk errors return immediately; a persistent owner still fails.
fn retry_windows_sharing_violation<T>(
    mut operation: impl FnMut() -> std::io::Result<T>,
) -> std::io::Result<T> {
    let mut retries = 0;
    loop {
        match operation() {
            Err(error) if cfg!(windows) && error.raw_os_error() == Some(32) && retries < 10 => {
                retries += 1;
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
            outcome => return outcome,
        }
    }
}

/// Write `migration_failed.json` under app data when migrations fail.
///
/// # Errors
///
/// Returns an error when the marker file cannot be written.
pub fn write_migration_failed_marker(app_data_dir: &Path, message: &str) -> Result<(), String> {
    let path = app_data_dir.join("migration_failed.json");
    let body = serde_json::json!({
        "failed_at": chrono::Utc::now().to_rfc3339(),
        "message": message,
    });
    let bytes = serde_json::to_vec_pretty(&body)
        .map_err(|error| format!("serialize migration failure marker: {error}"))?;
    super::file_commit::atomic_write(&path, &bytes)
        .map_err(|error| format!("write {}: {error}", path.display()))
}

/// Caller has closed the pool. Preserve the primary error before adding recovery evidence.
pub(crate) fn record_migration_failure(
    db_file: &Path,
    backup: Option<&Path>,
    app_data_dir: &Path,
    error: crate::error::AppError,
) -> crate::error::AppError {
    use crate::error::AppError;
    let mut message = match error {
        AppError::DbMigrationFailed(message) => message,
        other => other.to_string(),
    };
    let recovery = match backup {
        Some(path) => match restore_db_from_backup(db_file, path) {
            Ok(()) => format!("Backup restored from {}", path.display()),
            Err(error) => {
                tracing::error!(target: "oclive_migrate", %error, "migration backup restore failed");
                format!("Backup restore failed: {error}")
            }
        },
        None => "Backup restore not attempted: no pre-migration backup".into(),
    };
    message.push_str(&format!("\n{recovery}"));
    if let Err(error) = write_migration_failed_marker(app_data_dir, &message) {
        tracing::error!(target: "oclive_migrate", %error, "migration failure marker write failed");
        message.push_str(&format!("\nMigration failure marker write failed: {error}"));
    }
    AppError::DbMigrationFailed(message)
}

fn migration_checksum(sql: &str) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(sql.as_bytes());
    hasher.finalize().to_vec()
}

/// Return whether at least one bundled migration has not been applied successfully.
///
/// # Errors
///
/// Returns an error when migration files or the migration ledger cannot be read.
pub async fn has_pending_sql_migrations(
    db: &SqlitePool,
    migrations_dir: &Path,
) -> Result<bool, String> {
    let ledger_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = '_sqlx_migrations'",
    )
    .fetch_one(db)
    .await
    .map_err(|e| e.to_string())?;
    if ledger_exists == 0 {
        return Ok(true);
    }

    let applied: HashSet<i64> =
        sqlx::query_scalar::<_, i64>("SELECT version FROM _sqlx_migrations WHERE success = 1")
            .fetch_all(db)
            .await
            .map_err(|e| e.to_string())?
            .into_iter()
            .collect();
    let entries = std::fs::read_dir(migrations_dir)
        .map_err(|e| format!("read migrations dir {}: {e}", migrations_dir.display()))?;
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.extension().is_none_or(|ext| ext != "sql") {
            continue;
        }
        let file_name = entry.file_name().to_string_lossy().into_owned();
        let version: i64 = file_name
            .split('_')
            .next()
            .and_then(|part| part.parse().ok())
            .ok_or_else(|| format!("migration file name must start with version: {file_name}"))?;
        if !applied.contains(&version) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Apply `migrations/*.sql` in lexical order; compatible with existing `_sqlx_migrations` rows.
///
/// # Errors
///
/// Returns an error when the migrations directory cannot be read or a statement fails.
pub async fn run_sql_migrations(db: &SqlitePool, migrations_dir: &Path) -> Result<(), String> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS _sqlx_migrations (
            version BIGINT PRIMARY KEY,
            description TEXT NOT NULL,
            installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            success BOOLEAN NOT NULL,
            checksum BLOB NOT NULL,
            execution_time BIGINT NOT NULL
        )",
    )
    .execute(db)
    .await
    .map_err(|e| e.to_string())?;

    let mut entries: Vec<_> = std::fs::read_dir(migrations_dir)
        .map_err(|e| format!("read migrations dir {}: {e}", migrations_dir.display()))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "sql"))
        .collect();
    entries.sort_by_key(|e| e.file_name());

    let applied_rows: Vec<(i64, Vec<u8>)> =
        sqlx::query_as("SELECT version, checksum FROM _sqlx_migrations WHERE success = 1")
            .fetch_all(db)
            .await
            .map_err(|e| e.to_string())?;
    let applied_checksums: HashMap<i64, Vec<u8>> = applied_rows.into_iter().collect();

    for entry in entries {
        let file_name = entry.file_name().to_string_lossy().into_owned();
        let version: i64 = file_name
            .split('_')
            .next()
            .and_then(|p| p.parse().ok())
            .ok_or_else(|| format!("migration file name must start with version: {file_name}"))?;

        let sql = std::fs::read_to_string(entry.path())
            .map_err(|e| format!("read {}: {e}", entry.path().display()))?;
        let checksum = migration_checksum(&sql);

        if let Some(stored) = applied_checksums.get(&version) {
            if stored != &checksum {
                tracing::warn!(
                    target: "oclive_migrate",
                    version = version,
                    file = %file_name,
                    "applied migration checksum drift detected (file changed after install)"
                );
            }
            continue;
        }

        let started = std::time::Instant::now();
        let mut tx = db.begin().await.map_err(|e| e.to_string())?;
        // SQLite parses the entire batch, including trigger bodies and quoted
        // semicolons. Keep DDL and the migration ledger in the same transaction.
        (&mut *tx)
            .execute(sql.as_str())
            .await
            .map_err(|e| format!("migration {file_name}: {e}"))?;
        let elapsed_ms = started.elapsed().as_millis() as i64;
        sqlx::query(
            "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time)
             VALUES (?, ?, 1, ?, ?)",
        )
        .bind(version)
        .bind(&file_name)
        .bind(&checksum)
        .bind(elapsed_ms)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
        tx.commit().await.map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn migration_failure_restores_real_snapshot_and_records_primary_error() {
        let temp = tempfile::tempdir().unwrap();
        let database = temp.path().join("app.db");
        let pool = crate::infrastructure::sqlite_pool::connect_file(&database)
            .await
            .unwrap();
        sqlx::query("CREATE TABLE original(value TEXT)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO original VALUES ('kept')")
            .execute(&pool)
            .await
            .unwrap();
        let backup = backup_db_file(&pool, &database, temp.path()).await.unwrap();
        let migrations = temp.path().join("fixture-migrations");
        std::fs::create_dir(&migrations).unwrap();
        std::fs::write(migrations.join("001_invalid.sql"), "THIS IS NOT VALID SQL;").unwrap();
        let original_error = run_sql_migrations(&pool, &migrations).await.unwrap_err();
        pool.close().await;
        let error = record_migration_failure(
            &database,
            Some(&backup),
            temp.path(),
            crate::error::AppError::DbMigrationFailed(original_error.clone()),
        );
        assert_eq!(error.code(), "DB_MIGRATION_FAILED");
        assert!(error.to_string().contains(&original_error));
        assert!(error.to_string().contains("Backup restored"), "{error}");
        let marker: serde_json::Value = serde_json::from_slice(
            &std::fs::read(temp.path().join("migration_failed.json")).unwrap(),
        )
        .unwrap();
        assert!(marker["message"]
            .as_str()
            .unwrap()
            .contains(&original_error));
        assert!(marker["message"]
            .as_str()
            .unwrap()
            .contains("Backup restored"));
        let restored = crate::infrastructure::sqlite_pool::connect_file(&database)
            .await
            .unwrap();
        let value: String = sqlx::query_scalar("SELECT value FROM original")
            .fetch_one(&restored)
            .await
            .unwrap();
        assert_eq!(value, "kept");
        restored.close().await;
    }

    #[test]
    fn migration_failure_reports_missing_backup_without_touching_database_or_wal() {
        let temp = tempfile::tempdir().unwrap();
        let database = temp.path().join("app.db");
        std::fs::write(&database, "retained database").unwrap();
        std::fs::write(temp.path().join("app.db-wal"), "retained wal").unwrap();
        let error = record_migration_failure(
            &database,
            Some(&temp.path().join("missing.db")),
            temp.path(),
            crate::error::AppError::DbMigrationFailed("fixture primary failure".into()),
        );
        assert!(error
            .to_string()
            .contains("fixture primary failure\nBackup restore failed"));
        assert_eq!(std::fs::read(&database).unwrap(), b"retained database");
        assert_eq!(
            std::fs::read(temp.path().join("app.db-wal")).unwrap(),
            b"retained wal"
        );
        let marker = std::fs::read_to_string(temp.path().join("migration_failed.json")).unwrap();
        assert!(marker.contains("Backup restore failed"));
    }

    #[test]
    fn migration_failure_reports_restore_and_marker_errors_without_losing_primary() {
        let temp = tempfile::tempdir().unwrap();
        let database = temp.path().join("database-is-a-directory");
        std::fs::create_dir(&database).unwrap();
        let backup = temp.path().join("backup.db");
        std::fs::write(&backup, "backup retained").unwrap();
        let invalid_app_data = temp.path().join("data-is-a-file");
        std::fs::write(&invalid_app_data, "unchanged").unwrap();
        let error = record_migration_failure(
            &database,
            Some(&backup),
            &invalid_app_data,
            crate::error::AppError::DbMigrationFailed("fixture primary failure".into()),
        );
        assert_eq!(error.code(), "DB_MIGRATION_FAILED");
        let message = error.to_string();
        assert!(message.contains("fixture primary failure\nBackup restore failed"));
        assert!(message.contains("Migration failure marker write failed"));
        assert_eq!(std::fs::read(backup).unwrap(), b"backup retained");
        assert_eq!(std::fs::read(invalid_app_data).unwrap(), b"unchanged");
    }

    #[test]
    fn migration_failure_without_backup_is_explicit_and_marker_keeps_existing_shape() {
        let temp = tempfile::tempdir().unwrap();
        let error = record_migration_failure(
            &temp.path().join("app.db"),
            None,
            temp.path(),
            crate::error::AppError::DbMigrationFailed("primary".into()),
        );
        assert!(error.to_string().contains("no pre-migration backup"));
        let marker: serde_json::Value = serde_json::from_slice(
            &std::fs::read(temp.path().join("migration_failed.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(marker.as_object().unwrap().len(), 2);
        assert!(marker["failed_at"].is_string());
        assert!(marker["message"].as_str().unwrap().starts_with("primary\n"));
    }

    #[cfg(windows)]
    #[test]
    fn restore_waits_for_transient_windows_wal_sharing_violation() {
        use std::os::windows::fs::OpenOptionsExt;
        let temp = tempfile::tempdir().unwrap();
        let database = temp.path().join("app.db");
        let backup = temp.path().join("backup.db");
        let wal = temp.path().join("app.db-wal");
        std::fs::write(&database, "before").unwrap();
        std::fs::write(&backup, "after").unwrap();
        std::fs::write(&wal, "old wal").unwrap();
        let held = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .share_mode(0)
            .open(&wal)
            .unwrap();
        assert_eq!(
            std::fs::remove_file(&wal).unwrap_err().raw_os_error(),
            Some(32)
        );
        let release = std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(60));
            drop(held);
        });
        restore_db_from_backup(&database, &backup).unwrap();
        release.join().unwrap();
        assert_eq!(std::fs::read(&database).unwrap(), b"after");
        assert!(!wal.exists());
    }

    #[cfg(windows)]
    #[test]
    fn restore_reports_persistent_windows_lock_and_keeps_database_and_backup() {
        use std::os::windows::fs::OpenOptionsExt;
        let temp = tempfile::tempdir().unwrap();
        let database = temp.path().join("app.db");
        let backup = temp.path().join("backup.db");
        let wal = temp.path().join("app.db-wal");
        std::fs::write(&database, "before").unwrap();
        std::fs::write(&backup, "after").unwrap();
        std::fs::write(&wal, "old wal").unwrap();
        let _held = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .share_mode(0)
            .open(&wal)
            .unwrap();
        let error = restore_db_from_backup(&database, &backup).unwrap_err();
        assert!(error.contains("remove stale SQLite sidecar"));
        assert_eq!(std::fs::read(&database).unwrap(), b"before");
        assert_eq!(std::fs::read(&backup).unwrap(), b"after");
    }

    #[test]
    fn find_migrations_dir_finds_embedded_or_monorepo() {
        let dir = find_migrations_dir().expect("migrations dir");
        assert!(is_migrations_dir(&dir));
        assert!(
            dir.ends_with("migrations"),
            "expected .../migrations, got {}",
            dir.display()
        );
    }

    #[test]
    fn bundled_migrations_are_discovered_without_a_monorepo() {
        let temp = tempfile::tempdir().expect("tempdir");
        let migrations = temp.path().join("resources").join("migrations");
        std::fs::create_dir_all(&migrations).expect("create migrations");
        std::fs::write(migrations.join("001_init.sql"), "CREATE TABLE t (id INT);")
            .expect("write migration");

        let candidates = bundled_migration_candidates(&[temp.path().to_path_buf()]);
        let found = candidates
            .into_iter()
            .find(|candidate| is_migrations_dir(candidate));
        assert_eq!(found.as_deref(), Some(migrations.as_path()));
    }

    #[test]
    fn bundled_migrations_win_over_a_newer_build_checkout() {
        let temp = tempfile::tempdir().expect("tempdir");
        let bundle = temp.path().join("installed");
        let shipped = bundle.join("resources").join("migrations");
        let checkout = temp.path().join("checkout").join("migrations");
        for path in [&shipped, &checkout] {
            std::fs::create_dir_all(path).expect("mkdir");
        }
        std::fs::write(shipped.join("039_old.sql"), "CREATE TABLE old(id INT);")
            .expect("shipped SQL");
        std::fs::write(checkout.join("040_new.sql"), "CREATE TABLE new(id INT);")
            .expect("new checkout SQL");

        let found = local_migration_candidates(&[bundle], checkout)
            .into_iter()
            .find(|candidate| is_migrations_dir(candidate));
        assert_eq!(found.as_deref(), Some(shipped.as_path()));
    }

    #[test]
    fn migration_discovery_places_executable_before_working_directory() {
        let exe = std::env::current_exe().expect("test executable");
        let parent = exe.parent().expect("executable directory");
        let anchors = migration_discovery_anchors();
        assert_eq!(anchors.first().map(PathBuf::as_path), Some(parent));
    }

    #[test]
    fn executable_bundle_wins_over_a_different_working_directory_bundle() {
        let temp = tempfile::tempdir().expect("tempdir");
        let executable = temp.path().join("installed");
        let cwd = temp.path().join("other-build");
        let shipped = executable.join("resources").join("migrations");
        let unrelated = cwd.join("migrations");
        for path in [&shipped, &unrelated] {
            std::fs::create_dir_all(path).expect("mkdir");
            std::fs::write(path.join("001_init.sql"), "CREATE TABLE t(id INT);").expect("SQL");
        }
        let found = local_migration_candidates(&[executable, cwd], temp.path().join("checkout"))
            .into_iter()
            .find(|candidate| is_migrations_dir(candidate));
        assert_eq!(found.as_deref(), Some(shipped.as_path()));
    }

    #[test]
    fn development_without_a_bundle_still_finds_build_checkout() {
        let temp = tempfile::tempdir().expect("tempdir");
        let checkout = temp.path().join("checkout").join("migrations");
        std::fs::create_dir_all(&checkout).expect("mkdir");
        std::fs::write(checkout.join("001_init.sql"), "CREATE TABLE t(id INT);")
            .expect("checkout SQL");

        let found = local_migration_candidates(&[temp.path().join("no-bundle")], checkout.clone())
            .into_iter()
            .find(|candidate| is_migrations_dir(candidate));
        assert_eq!(found.as_deref(), Some(checkout.as_path()));
    }

    #[test]
    fn checksum_is_sha256() {
        let sql = "CREATE TABLE t (id INT);";
        let digest = migration_checksum(sql);
        assert_eq!(digest.len(), 32);
    }

    #[tokio::test]
    async fn pending_migrations_are_detected_before_apply_only() {
        let pool = crate::infrastructure::sqlite_pool::connect_memory()
            .await
            .expect("in-memory pool");
        let dir = find_migrations_dir().expect("migrations dir");
        assert!(has_pending_sql_migrations(&pool, &dir)
            .await
            .expect("pending before apply"));
        run_sql_migrations(&pool, &dir).await.expect("apply all");
        assert!(!has_pending_sql_migrations(&pool, &dir)
            .await
            .expect("pending after apply"));
    }

    #[tokio::test]
    async fn backup_includes_committed_wal_rows_and_passes_integrity_check() {
        let temp = tempfile::tempdir().expect("tempdir");
        let db_path = temp.path().join("app.db");
        let pool = crate::infrastructure::sqlite_pool::connect_file(&db_path)
            .await
            .expect("file pool");
        sqlx::query("CREATE TABLE sample (id INTEGER PRIMARY KEY, value TEXT NOT NULL)")
            .execute(&pool)
            .await
            .expect("create table");
        sqlx::query("INSERT INTO sample (value) VALUES ('committed in WAL')")
            .execute(&pool)
            .await
            .expect("insert row");

        let backup = backup_db_file(&pool, &db_path, temp.path())
            .await
            .expect("consistent backup");
        let backup_pool = crate::infrastructure::sqlite_pool::connect_file(&backup)
            .await
            .expect("backup pool");
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sample")
            .fetch_one(&backup_pool)
            .await
            .expect("backup count");
        let integrity: String = sqlx::query_scalar("PRAGMA integrity_check")
            .fetch_one(&backup_pool)
            .await
            .expect("backup integrity");
        assert_eq!(count, 1);
        assert_eq!(integrity, "ok");
    }

    #[test]
    fn restore_removes_stale_wal_sidecars() {
        let temp = tempfile::tempdir().expect("tempdir");
        let db_path = temp.path().join("app.db");
        let backup = temp.path().join("clean.db");
        std::fs::write(&db_path, b"old").expect("old db");
        std::fs::write(&backup, b"clean").expect("clean backup");
        std::fs::write(temp.path().join("app.db-wal"), b"stale wal").expect("wal");
        std::fs::write(temp.path().join("app.db-shm"), b"stale shm").expect("shm");

        restore_db_from_backup(&db_path, &backup).expect("restore");

        assert_eq!(std::fs::read(&db_path).expect("restored db"), b"clean");
        assert!(!temp.path().join("app.db-wal").exists());
        assert!(!temp.path().join("app.db-shm").exists());
    }

    async fn emotion_source_column_count(pool: &sqlx::SqlitePool) -> i64 {
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM pragma_table_info('chat_messages') WHERE name = 'emotion_source'",
        )
        .fetch_one(pool)
        .await
        .expect("pragma table_info")
    }

    #[tokio::test]
    async fn migration_039_adds_emotion_source_on_fresh_db() {
        let pool = crate::infrastructure::sqlite_pool::connect_memory()
            .await
            .expect("in-memory pool");
        let dir = find_migrations_dir().expect("migrations dir");
        run_sql_migrations(&pool, &dir).await.expect("apply all");
        assert_eq!(emotion_source_column_count(&pool).await, 1);
    }

    #[tokio::test]
    async fn migration_039_upgrades_db_from_038_keeping_old_rows_null() {
        let pool = crate::infrastructure::sqlite_pool::connect_memory()
            .await
            .expect("in-memory pool");
        let full_dir = find_migrations_dir().expect("migrations dir");
        let tmp = tempfile::tempdir().expect("temp dir");
        for entry in std::fs::read_dir(&full_dir).expect("read dir") {
            let path = entry.expect("entry").path();
            let name = path
                .file_name()
                .expect("name")
                .to_string_lossy()
                .into_owned();
            if !name.ends_with(".sql")
                || name
                    .split('_')
                    .next()
                    .and_then(|v| v.parse::<u32>().ok())
                    .is_none_or(|v| v >= 39)
            {
                continue;
            }
            std::fs::copy(&path, tmp.path().join(&name)).expect("copy migration");
        }
        run_sql_migrations(&pool, tmp.path())
            .await
            .expect("apply 001-038");
        assert_eq!(emotion_source_column_count(&pool).await, 0);

        // simulate a pre-039 row
        sqlx::query(
            "INSERT INTO chat_sessions (session_id, role_id, scene_id, created_at, updated_at)
             VALUES ('old', 'old', 'default', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
        )
        .execute(&pool)
        .await
        .expect("insert old session");
        sqlx::query(
            "INSERT INTO chat_messages (id, session_id, turn_index, sender, content, created_at)
             VALUES ('old-msg', 'old', 0, 'assistant', 'hi', '2026-01-01T00:00:00Z')",
        )
        .execute(&pool)
        .await
        .expect("insert old message");

        run_sql_migrations(&pool, &full_dir)
            .await
            .expect("apply 039");
        assert_eq!(emotion_source_column_count(&pool).await, 1);
        let applied: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _sqlx_migrations WHERE version = 39 AND success = 1",
        )
        .fetch_one(&pool)
        .await
        .expect("migration row");
        assert_eq!(applied, 1);
        let old_source: Option<String> =
            sqlx::query_scalar("SELECT emotion_source FROM chat_messages WHERE id = 'old-msg'")
                .fetch_one(&pool)
                .await
                .expect("old row");
        assert!(
            old_source.is_none(),
            "pre-039 rows keep NULL emotion_source"
        );
    }
    #[tokio::test]
    async fn native_batch_preserves_trigger_literals_comments_and_rolls_back_failure() {
        let pool = crate::infrastructure::sqlite_pool::connect_memory()
            .await
            .expect("pool");
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::write(tmp.path().join("001_batch.sql"),
            "-- comment; ignored\nCREATE TABLE source(v TEXT); CREATE TABLE audit(v TEXT);\nCREATE TRIGGER capture AFTER INSERT ON source BEGIN INSERT INTO audit VALUES ('semi;--literal'); INSERT INTO audit VALUES (NEW.v); END;\nINSERT INTO source VALUES ('value');").expect("write");
        run_sql_migrations(&pool, tmp.path())
            .await
            .expect("native batch");
        let values: Vec<String> = sqlx::query_scalar("SELECT v FROM audit ORDER BY rowid")
            .fetch_all(&pool)
            .await
            .expect("audit");
        assert_eq!(values, vec!["semi;--literal", "value"]);
        std::fs::write(
            tmp.path().join("002_failure.sql"),
            "CREATE TABLE rollback_probe(v TEXT); INSERT INTO missing_table VALUES (1);",
        )
        .expect("write");
        assert!(run_sql_migrations(&pool, tmp.path()).await.is_err());
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM sqlite_master WHERE name='rollback_probe'")
                .fetch_one(&pool)
                .await
                .expect("rollback");
        assert_eq!(count, 0);
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations WHERE version=2")
                .fetch_one(&pool)
                .await
                .expect("ledger");
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn migration_040_fresh_and_upgrade_keep_history_and_install_receipt_triggers() {
        let full = find_migrations_dir().expect("migrations");
        for upgrade in [false, true] {
            let pool = crate::infrastructure::sqlite_pool::connect_memory()
                .await
                .expect("pool");
            let tmp = tempfile::tempdir().expect("tempdir");
            if upgrade {
                for entry in std::fs::read_dir(&full).expect("read") {
                    let path = entry.expect("entry").path();
                    let name = path.file_name().expect("name").to_string_lossy();
                    if name.ends_with(".sql")
                        && name
                            .split('_')
                            .next()
                            .and_then(|v| v.parse::<u32>().ok())
                            .is_some_and(|v| v < 40)
                    {
                        std::fs::copy(&path, tmp.path().join(name.as_ref())).expect("copy");
                    }
                }
                run_sql_migrations(&pool, tmp.path())
                    .await
                    .expect("pre-040");
                pool.execute("INSERT INTO chat_sessions(session_id,role_id,scene_id,created_at,updated_at) VALUES('old','old','default','t','t'); INSERT INTO chat_messages(id,session_id,turn_index,sender,content,created_at) VALUES('old-msg','old',0,'assistant','keep me','t');").await.expect("old history");
            }
            run_sql_migrations(&pool, &full)
                .await
                .expect("040 production runner");
            run_sql_migrations(&pool, &full)
                .await
                .expect("idempotent migration");
            let installed: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM _sqlx_migrations WHERE version=40 AND success=1",
            )
            .fetch_one(&pool)
            .await
            .expect("ledger");
            assert_eq!(installed, 1);
            let triggers: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sqlite_master WHERE type='trigger' AND name IN ('chat_request_message_deleted','chat_request_session_deleted')").fetch_one(&pool).await.expect("triggers");
            assert_eq!(triggers, 2);
            if upgrade {
                let content: String =
                    sqlx::query_scalar("SELECT content FROM chat_messages WHERE id='old-msg'")
                        .fetch_one(&pool)
                        .await
                        .expect("history retained");
                assert_eq!(content, "keep me");
                pool.execute("INSERT INTO chat_request_receipts(scope_key,request_id,payload_sha256,session_namespace,role_id,scene_id,status,response_json,assistant_message_id) VALUES('scope','id','hash','old','old','default','completed','cached','old-msg'); DELETE FROM chat_messages WHERE id='old-msg';").await.expect("delete");
                let result: (String, Option<String>) =
                    sqlx::query_as("SELECT status,response_json FROM chat_request_receipts")
                        .fetch_one(&pool)
                        .await
                        .expect("receipt");
                assert_eq!(result, ("unconfirmed".into(), None));
            }
            pool.close().await;
        }
    }
}

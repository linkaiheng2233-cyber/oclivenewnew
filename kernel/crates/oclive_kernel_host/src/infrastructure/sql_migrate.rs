//! Runtime SQLite migrations without the `sqlx` umbrella `migrate` feature (avoids mysql/postgres in the lockfile).

use oclive_kernel_runtime::{find_monorepo_root, ENV_ROLES_DIR};
use sha2::{Digest, Sha256};
use sqlx::sqlite::SqlitePool;
use std::collections::HashMap;
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
    if let Ok(cwd) = std::env::current_dir() {
        anchors.push(cwd);
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            anchors.push(parent.to_path_buf());
        }
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

/// Resolve migrations directory for runtime apply.
///
/// Order: `OCLIVE_MIGRATIONS_DIR` → compile-time embed → monorepo
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

    let embedded = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("migrations");
    if is_migrations_dir(&embedded) {
        return Ok(embedded);
    }
    tried.push(format!(
        "embedded {} (missing or no .sql)",
        embedded.display()
    ));

    let anchors = migration_discovery_anchors();
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

/// Copy `db_file` to `app_data/app.db.bak.{unix_secs}` before migrations (file DB only).
///
/// # Errors
///
/// Returns an error when the backup copy fails.
pub fn backup_db_file(db_file: &Path, app_data_dir: &Path) -> Result<PathBuf, String> {
    if !db_file.is_file() {
        return Ok(PathBuf::new());
    }
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let dest = app_data_dir.join(format!("app.db.bak.{ts}"));
    std::fs::create_dir_all(app_data_dir)
        .map_err(|e| format!("backup mkdir {}: {e}", app_data_dir.display()))?;
    std::fs::copy(db_file, &dest)
        .map_err(|e| format!("backup {} -> {}: {e}", db_file.display(), dest.display()))?;
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
    if !backup.is_file() {
        return Ok(());
    }
    if let Some(parent) = db_file.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("restore mkdir {}: {e}", parent.display()))?;
    }
    std::fs::copy(backup, db_file)
        .map_err(|e| format!("restore {} <- {}: {e}", db_file.display(), backup.display()))?;
    Ok(())
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
    std::fs::write(
        &path,
        serde_json::to_string_pretty(&body).unwrap_or_default(),
    )
    .map_err(|e| format!("write {}: {e}", path.display()))
}

fn migration_checksum(sql: &str) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(sql.as_bytes());
    hasher.finalize().to_vec()
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
        for statement in split_sql_statements(&sql) {
            sqlx::query(&statement)
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("migration {file_name}: {e}\nSQL: {statement}"))?;
        }
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

/// Strip `--` line comments first, then split on `;`.
///
/// Naively splitting on `;` would break when a `;` appears in a comment line
/// (e.g. `-- foo; bar.`) because the loose tail would be sent to SQLite as a
/// statement and fail. We strip comments line-by-line first; we do **not**
/// attempt to handle string-literal `;` because our migrations don't use them
/// in DDL — if that ever changes, replace with a real tokenizer.
fn split_sql_statements(sql: &str) -> Vec<String> {
    let stripped: String = sql
        .lines()
        .map(strip_line_comment)
        .collect::<Vec<_>>()
        .join("\n");

    stripped
        .split(';')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

/// Drop everything after `--` on a single line; preserves leading whitespace
/// so multi-line statements still parse correctly.
fn strip_line_comment(line: &str) -> String {
    match line.find("--") {
        Some(idx) => line[..idx].trim_end().to_string(),
        None => line.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn split_drops_pure_comment_line() {
        let sql = "-- only a comment";
        assert!(split_sql_statements(sql).is_empty());
    }

    #[test]
    fn split_handles_semicolon_inside_comment() {
        let sql =
            "-- chat history (kernel-owned; independent from memory).\nCREATE TABLE foo (id INT);";
        let stmts = split_sql_statements(sql);
        assert_eq!(stmts.len(), 1);
        assert!(stmts[0].starts_with("CREATE TABLE foo"));
    }

    #[test]
    fn split_keeps_trailing_inline_comment() {
        let sql = "CREATE TABLE foo (id INT); -- trailing\nCREATE TABLE bar (id INT);";
        let stmts = split_sql_statements(sql);
        assert_eq!(stmts.len(), 2);
        assert!(stmts[0].contains("foo"));
        assert!(stmts[1].contains("bar"));
    }

    #[test]
    fn split_preserves_multiline_statement() {
        let sql = "CREATE TABLE foo (\n  id INT,\n  name TEXT\n);";
        let stmts = split_sql_statements(sql);
        assert_eq!(stmts.len(), 1);
        assert!(stmts[0].contains("name TEXT"));
    }

    #[test]
    fn checksum_is_sha256() {
        let sql = "CREATE TABLE t (id INT);";
        let digest = migration_checksum(sql);
        assert_eq!(digest.len(), 32);
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
            if !name.ends_with(".sql") || name.starts_with("039_") {
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
}

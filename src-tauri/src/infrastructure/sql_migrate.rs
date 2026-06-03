//! Runtime SQLite migrations without the `sqlx` umbrella `migrate` feature (avoids mysql/postgres in the lockfile).

use sqlx::sqlite::SqlitePool;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

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
    std::fs::write(&path, serde_json::to_string_pretty(&body).unwrap_or_default())
        .map_err(|e| format!("write {}: {e}", path.display()))
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

    // Avoid one round trip per migration file.
    let applied_versions: HashSet<i64> = sqlx::query_scalar(
        "SELECT version FROM _sqlx_migrations WHERE success = 1",
    )
    .fetch_all(db)
    .await
    .map_err(|e| e.to_string())?
    .into_iter()
    .collect();

    for entry in entries {
        let file_name = entry.file_name().to_string_lossy().into_owned();
        let version: i64 = file_name
            .split('_')
            .next()
            .and_then(|p| p.parse().ok())
            .ok_or_else(|| format!("migration file name must start with version: {file_name}"))?;

        if applied_versions.contains(&version) {
            continue;
        }

        let sql = std::fs::read_to_string(entry.path())
            .map_err(|e| format!("read {}: {e}", entry.path().display()))?;
        let started = std::time::Instant::now();
        for statement in split_sql_statements(&sql) {
            sqlx::query(&statement)
                .execute(db)
                .await
                .map_err(|e| format!("migration {file_name}: {e}\nSQL: {statement}"))?;
        }
        let elapsed_ms = started.elapsed().as_millis() as i64;
        let checksum: Vec<u8> = sql.as_bytes().to_vec();
        sqlx::query(
            "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time)
             VALUES (?, ?, 1, ?, ?)",
        )
        .bind(version)
        .bind(&file_name)
        .bind(checksum)
        .bind(elapsed_ms)
        .execute(db)
        .await
        .map_err(|e| e.to_string())?;
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
    fn split_drops_pure_comment_line() {
        let sql = "-- only a comment";
        assert!(split_sql_statements(sql).is_empty());
    }

    #[test]
    fn split_handles_semicolon_inside_comment() {
        let sql = "-- chat history (kernel-owned; independent from memory).\nCREATE TABLE foo (id INT);";
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
}

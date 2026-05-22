//! Runtime SQLite migrations without the `sqlx` umbrella `migrate` feature (avoids mysql/postgres in the lockfile).

use sqlx::sqlite::SqlitePool;
use std::path::Path;

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

    for entry in entries {
        let file_name = entry.file_name().to_string_lossy().into_owned();
        let version: i64 = file_name
            .split('_')
            .next()
            .and_then(|p| p.parse().ok())
            .ok_or_else(|| format!("migration file name must start with version: {file_name}"))?;

        let applied: Option<(i64,)> = sqlx::query_as(
            "SELECT version FROM _sqlx_migrations WHERE version = ? AND success = 1 LIMIT 1",
        )
        .bind(version)
        .fetch_optional(db)
        .await
        .map_err(|e| e.to_string())?;
        if applied.is_some() {
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

fn split_sql_statements(sql: &str) -> Vec<String> {
    sql.split(';')
        .map(str::trim)
        .filter(|s| !s.is_empty() && !s.lines().all(|l| l.trim().is_empty() || l.trim().starts_with("--")))
        .map(|s| s.to_string())
        .collect()
}

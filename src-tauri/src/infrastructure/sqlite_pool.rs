//! Shared SQLite pool options for desktop persistence (WAL, busy timeout, pool size).

use sqlx::sqlite::{
    SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions, SqliteSynchronous,
};
use std::path::Path;
use std::time::Duration;

const POOL_MAX_CONNECTIONS: u32 = 8;

/// Open an on-disk SQLite pool tuned for desktop concurrency (WAL + `NORMAL` sync).
///
/// # Errors
///
/// Returns [`sqlx::Error`] when the pool cannot be opened or configured.
pub async fn connect_file(path: impl AsRef<Path>) -> Result<SqlitePool, sqlx::Error> {
    let opts = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .busy_timeout(Duration::from_secs(5))
        .pragma("temp_store", "MEMORY");
    SqlitePoolOptions::new()
        .min_connections(1)
        .max_connections(POOL_MAX_CONNECTIONS)
        .acquire_timeout(Duration::from_secs(10))
        .connect_with(opts)
        .await
}

/// In-memory pool for tests and headless fixtures (WAL pragmas not applicable).
///
/// # Errors
///
/// Returns [`sqlx::Error`] when the in-memory pool cannot be created.
pub async fn connect_memory() -> Result<SqlitePool, sqlx::Error> {
    SqlitePoolOptions::new()
        .min_connections(1)
        .max_connections(POOL_MAX_CONNECTIONS)
        .acquire_timeout(Duration::from_secs(10))
        .connect("sqlite::memory:")
        .await
}

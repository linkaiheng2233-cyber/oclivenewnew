//! Shared in-memory SQLite fixtures for unit tests.

use crate::infrastructure::db::DbManager;
use crate::infrastructure::sql_migrate;
use crate::infrastructure::sqlite_pool;
use sqlx::SqlitePool;

/// Open an in-memory pool and apply all migrations (same runner as production; FK on).
///
/// # Panics
///
/// Panics if the in-memory pool or migration runner fails (test fixture only).
pub async fn connect_memory_migrated() -> SqlitePool {
    let pool = sqlite_pool::connect_memory()
        .await
        .expect("in-memory sqlite pool");
    let migrations_dir = sql_migrate::find_migrations_dir().expect("migrations dir");
    sql_migrate::run_sql_migrations(&pool, &migrations_dir)
        .await
        .expect("apply migrations");
    pool
}

/// In-memory [`DbManager`] with schema migrated.
pub async fn mem_db_manager() -> DbManager {
    DbManager::new(connect_memory_migrated().await)
}

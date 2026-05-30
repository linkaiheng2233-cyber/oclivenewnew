//! Shared in-memory SQLite fixtures for unit tests.

use sqlx::SqlitePool;

use crate::infrastructure::db::DbManager;
use crate::infrastructure::sqlite_pool;

/// Open an in-memory pool and apply all migrations from `migrations/`.
pub async fn connect_memory_migrated() -> SqlitePool {
    let pool = sqlite_pool::connect_memory()
        .await
        .expect("in-memory sqlite pool");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("apply migrations");
    pool
}

/// In-memory [`DbManager`] with schema migrated.
pub async fn mem_db_manager() -> DbManager {
    DbManager::new(connect_memory_migrated().await)
}

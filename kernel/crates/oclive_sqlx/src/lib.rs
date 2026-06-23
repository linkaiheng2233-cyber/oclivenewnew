//! SQLite-only SQLx facade: re-exports the subset of the umbrella `sqlx` crate used by OCLive,
//! without pulling `sqlx-mysql` / `rsa` into `Cargo.lock`.

pub use sqlx_core::{
    error,
    executor::Executor,
    migrate,
    query::{query, query_with},
    query_as::{query_as, query_as_with},
    query_scalar::{query_scalar, query_scalar_with},
    row::Row,
    transaction::{Transaction, TransactionManager},
    Error, Result,
};

pub use sqlx_sqlite::{
    Sqlite, SqliteConnectOptions, SqliteConnection, SqliteJournalMode, SqlitePool,
    SqlitePoolOptions, SqliteRow, SqliteStatement, SqliteSynchronous, SqliteTransaction,
    SqliteTypeInfo, SqliteValue, SqliteValueRef,
};

/// SQLite driver module (matches `sqlx::sqlite` paths in call sites).
pub mod sqlite {
    pub use sqlx_sqlite::*;
}

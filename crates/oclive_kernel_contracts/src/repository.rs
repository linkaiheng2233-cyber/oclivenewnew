//! Data access ports (traits), implemented by the host `infrastructure`.

use async_trait::async_trait;
use oclive_kernel_types::{Memory, Result};

/// Persistence port for role-scoped long-term memories.
///
/// ## When to implement
///
/// - **Who**: the host persistence layer (e.g. `SqliteMemoryRepository`).
/// - **When**: when SQLite needs to be replaced or the storage backend swapped.
///
/// ## When not to implement
///
/// - Plugin authors usually do **not** implement this; for memory **retrieval** see [`MemoryRetrieval`](crate::MemoryRetrieval).
#[async_trait]
pub trait MemoryRepository: Send + Sync {
    /// Persists a single long-term memory and returns its ID.
    ///
    /// # Errors
    ///
    /// Returns `Err` on database I/O failure, constraint conflict, or serialization failure.
    ///
    /// # Panics
    ///
    /// Does not panic.
    async fn save_memory(&self, role_id: &str, content: &str, importance: f64) -> Result<String>;

    /// Loads the most recent memories for a role.
    ///
    /// # Errors
    ///
    /// Returns `Err` on database query failure or row deserialization failure.
    ///
    /// # Panics
    ///
    /// Does not panic.
    async fn load_memories(&self, role_id: &str, limit: i32) -> Result<Vec<Memory>>;

    /// Counts the number of memories under a role.
    ///
    /// # Errors
    ///
    /// Returns `Err` on database query failure.
    ///
    /// # Panics
    ///
    /// Does not panic.
    async fn count_memories(&self, role_id: &str) -> Result<i64>;

    /// Loads a role's memories with pagination.
    ///
    /// # Errors
    ///
    /// Returns `Err` on database query failure or row deserialization failure.
    ///
    /// # Panics
    ///
    /// Does not panic.
    async fn load_memories_paged(
        &self,
        role_id: &str,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<Memory>>;
}

/// Persistence port for role favorability scores.
///
/// ## When to implement
///
/// - **Who**: the host persistence layer (SQLite, etc.).
/// - **When**: when replacing favorability storage or adding new fields.
///
/// ## When not to implement
///
/// - Plugin / Remote backends generally do not implement this trait.
#[async_trait]
pub trait FavorabilityRepository: Send + Sync {
    /// Reads a role's current favorability.
    ///
    /// # Errors
    ///
    /// Returns `Err` on database query failure.
    ///
    /// # Panics
    ///
    /// Does not panic.
    async fn get(&self, role_id: &str) -> Result<Option<f64>>;

    /// Applies a delta change to a role's favorability.
    ///
    /// # Errors
    ///
    /// Returns `Err` on database read/write failure or update constraint conflict.
    ///
    /// # Panics
    ///
    /// Does not panic.
    async fn apply_delta(&self, role_id: &str, delta: f64) -> Result<()>;
}

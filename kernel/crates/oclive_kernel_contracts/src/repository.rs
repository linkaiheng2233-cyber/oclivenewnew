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

    /// Merge-save with keyword dedupe (same semantics as turn pipeline / bridge `update_memory`).
    ///
    /// # Errors
    ///
    /// Returns `Err` on database I/O failure.
    async fn save_memory_merged(
        &self,
        role_id: &str,
        content: &str,
        importance: f64,
        similarity_threshold: f64,
        scene_id: &str,
    ) -> Result<String>;

    /// Delete one memory row scoped to a role; returns whether a row was removed.
    ///
    /// # Errors
    ///
    /// Returns `Err` on database I/O failure.
    async fn delete_memory_for_role(&self, role_id: &str, memory_id: &str) -> Result<bool>;
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

/// Profile-mode mutable personality archive (relation transition / estrangement).
#[async_trait]
pub trait MutablePersonalityStore: Send + Sync {
    /// # Errors
    ///
    /// Returns `Err` on database query failure.
    async fn get_mutable_personality(&self, role_id: &str) -> Result<String>;

    /// # Errors
    ///
    /// Returns `Err` on database write failure.
    async fn set_mutable_personality(&self, role_id: &str, text: &str) -> Result<()>;

    /// Profile archive + seven-dim core/delta in one transaction (profile evolution path).
    ///
    /// # Errors
    ///
    /// Returns `Err` when either step fails; the transaction is rolled back.
    async fn apply_profile_evolution_atomic(
        &self,
        role_id: &str,
        mutable_text: &str,
        core_json: &str,
        delta_json: &str,
    ) -> Result<()>;
}

/// Relation identity + favorability persistence (estrangement / per-user relation stats).
#[async_trait]
pub trait RelationIdentityStore: MutablePersonalityStore + Send + Sync {
    /// # Errors
    ///
    /// Returns `Err` on database query failure.
    async fn get_last_interaction_at(
        &self,
        role_id: &str,
    ) -> Result<Option<chrono::DateTime<chrono::Utc>>>;

    /// # Errors
    ///
    /// Returns `Err` on database query failure.
    async fn get_favorability_for_identity(
        &self,
        role_id: &str,
        user_relation_key: &str,
    ) -> Result<Option<f64>>;

    /// # Errors
    ///
    /// Returns `Err` on database write failure.
    async fn set_identity_favorability_value(
        &self,
        role_id: &str,
        user_relation_key: &str,
        value: f64,
    ) -> Result<()>;

    /// # Errors
    ///
    /// Returns `Err` on database query failure.
    async fn get_relation_state_for_identity(
        &self,
        role_id: &str,
        user_relation_key: &str,
    ) -> Result<Option<String>>;

    /// # Errors
    ///
    /// Returns `Err` on database query failure.
    async fn get_relation_state(&self, role_id: &str) -> Result<Option<String>>;

    /// # Errors
    ///
    /// Returns `Err` on database write failure.
    async fn set_identity_relation_state(
        &self,
        role_id: &str,
        user_relation_key: &str,
        relation_state: &str,
    ) -> Result<()>;
}

/// Persisted `narrative_hint` for complex emotion (one-turn delayed Prompt injection).
#[async_trait]
pub trait ComplexEmotionHintStore: Send + Sync {
    /// # Errors
    ///
    /// Returns `Err` on database query failure.
    async fn get_complex_emotion_hint(&self, srid: &str) -> Result<Option<(String, String)>>;

    /// # Errors
    ///
    /// Returns `Err` on database write failure.
    async fn set_complex_emotion_hint(
        &self,
        srid: &str,
        narrative_hint: &str,
        updated_at: &str,
    ) -> Result<()>;

    /// # Errors
    ///
    /// Returns `Err` on database write failure.
    async fn delete_complex_emotion_hint(&self, srid: &str) -> Result<()>;
}

/// Immersive-mode virtual clock anchors and current virtual timestamp.
#[async_trait]
pub trait VirtualTimeStore: Send + Sync {
    /// # Errors
    ///
    /// Returns `Err` on database query failure.
    async fn get_virtual_time_anchors(&self, role_id: &str) -> Result<(i64, i64, i64)>;

    /// # Errors
    ///
    /// Returns `Err` on database write failure.
    async fn set_virtual_time_anchors(
        &self,
        role_id: &str,
        anchor_real_ms: i64,
        anchor_virtual_ms: i64,
    ) -> Result<()>;

    /// # Errors
    ///
    /// Returns `Err` on database write failure.
    async fn set_virtual_time_ms(&self, role_id: &str, ms: i64) -> Result<()>;
}

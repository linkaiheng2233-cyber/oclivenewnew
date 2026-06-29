//! Turn-thinking latch / ephemeral archive persistence port (decouples domain from `DbManager`).

use crate::error::Result;
use async_trait::async_trait;

/// Read/write `role_runtime` fields used by [`crate::domain::turn_thinking::update_turn_thinking_runtime_state`].
#[async_trait]
pub trait TurnThinkingStatePort: Send + Sync {
    /// # Errors
    ///
    /// Database write failures propagate as [`crate::error::AppError`].
    async fn set_deep_latch_active(&self, role_id: &str, active: bool) -> Result<()>;

    /// # Errors
    ///
    /// Database read failures propagate as [`crate::error::AppError`].
    async fn get_ephemeral_ttl_turns(&self, role_id: &str) -> Result<u32>;

    /// # Errors
    ///
    /// Database read failures propagate as [`crate::error::AppError`].
    async fn get_ephemeral_personality(&self, role_id: &str) -> Result<String>;

    /// # Errors
    ///
    /// Database write failures propagate as [`crate::error::AppError`].
    async fn set_ephemeral_personality(&self, role_id: &str, text: &str) -> Result<()>;

    /// # Errors
    ///
    /// Database write failures propagate as [`crate::error::AppError`].
    async fn set_ephemeral_ttl_turns(&self, role_id: &str, ttl: u32) -> Result<()>;
}

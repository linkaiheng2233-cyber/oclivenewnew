//! Database health probe port (decouples domain from `DbManager`).

use crate::error::Result;
use async_trait::async_trait;

/// Lightweight DB connectivity check for startup / HTTP health gates.
#[async_trait]
pub trait DbHealthPort: Send + Sync {
    /// # Errors
    ///
    /// Database connectivity failures propagate as [`crate::error::AppError`].
    async fn health_ping(&self) -> Result<()>;
}

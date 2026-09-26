//! Atomic admission and durable outcome storage for identified user turns.
use crate::error::Result;
use crate::models::dto::SendMessageResponse;
use async_trait::async_trait;

pub(crate) enum Receipt {
    Running,
    Completed(String),
    Unconfirmed,
}

#[async_trait]
pub(crate) trait ChatRequestStore: Send + Sync {
    /// Only the first successful claimant may enter the turn pipeline.
    async fn claim_chat_request(
        &self,
        scope: &str,
        id: &str,
        fingerprint: &str,
        session_namespace: &str,
        role_scene: (&str, &str),
    ) -> Result<bool>;
    async fn read_chat_request(&self, scope: &str, id: &str, fingerprint: &str) -> Result<Receipt>;
    async fn complete_chat_request(
        &self,
        scope: &str,
        id: &str,
        response: &SendMessageResponse,
    ) -> Result<()>;
    async fn mark_chat_request_unconfirmed(&self, scope: &str, id: &str) -> Result<()>;
}

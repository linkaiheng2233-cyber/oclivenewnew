//! Chat turn atomic DB persistence port (decouples domain from `DbManager`).

use crate::error::Result;
use crate::models::{Event, PersonalityVector};
use async_trait::async_trait;

/// Domain input for one atomic chat-turn DB transaction.
pub struct ChatTurnAtomicInput<'a> {
    pub role_id: &'a str,
    pub personality: &'a PersonalityVector,
    pub current_emotion: &'a str,
    pub relation_state: &'a str,
    pub user_relation_key: &'a str,
    pub favor_delta: f64,
    pub memory_content: &'a str,
    pub memory_importance: f64,
    pub memory_fifo_limit: i32,
    pub memory_similarity_threshold: f64,
    pub event: &'a Event,
    pub user_message: &'a str,
    pub bot_reply: &'a str,
    pub scene_id: &'a str,
}

/// Persists personality, emotion, relation, memory, and event in one transaction.
#[async_trait]
pub trait ChatTurnPersistencePort: Send + Sync {
    /// # Errors
    ///
    /// Database or validation failures.
    async fn apply_chat_turn_atomic(&self, input: ChatTurnAtomicInput<'_>) -> Result<f64>;
}

//! 情绪 / 事件 / 记忆策略 trait。

use oclive_kernel_types::{
    Emotion, EmotionResult, Event, EventType, PolicyContext, Result,
};

/// Maps analyzed user emotion into the role's displayed [`Emotion`].
pub trait EmotionPolicy: Send + Sync {
    fn resolve_current_emotion(&self, previous: Option<&str>, analyzed: &EmotionResult) -> Emotion;
}

/// Detects in-turn events and supplies impact/confidence weights per [`EventType`].
pub trait EventPolicy: Send + Sync {
    /// # Errors
    ///
    /// Returns an error when the policy cannot classify the message into an [`Event`].
    fn detect(&self, text: &str, user_emotion: &Emotion, bot_emotion: &Emotion) -> Result<Event>;
    fn impact(&self, event_type: &EventType) -> f64;
    fn confidence(&self, event_type: &EventType) -> f32;
}

/// Decides what to persist as long-term memory and with what importance.
pub trait MemoryPolicy: Send + Sync {
    fn build_memory_entry(&self, ctx: &PolicyContext<'_>) -> String;
    fn should_persist(&self, ctx: &PolicyContext<'_>) -> bool;
    fn importance(&self, ctx: &PolicyContext<'_>) -> f64;
    fn fifo_limit(&self) -> i32;
}

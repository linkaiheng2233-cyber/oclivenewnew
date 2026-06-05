//! Emotion / event / memory policy traits.

use oclive_kernel_types::{Emotion, EmotionResult, Event, EventType, PolicyContext, Result};

/// Maps analyzed user emotion into the role's displayed [`Emotion`].
///
/// ## When to implement
///
/// - **Who**: engine authors replacing the default personality/emotion mapping rules.
/// - **When**: when a custom "analyzed emotion → displayed role emotion" mapping is needed.
///
/// ## When not to implement
///
/// - When the builtin `DefaultEmotionPolicy` is used and its behavior already meets the requirements.
pub trait EmotionPolicy: Send + Sync {
    /// Maps the analyzed user emotion into the role's currently displayed emotion.
    ///
    /// # Errors
    ///
    /// None; this method does not return a `Result`.
    ///
    /// # Panics
    ///
    /// Does not panic.
    fn resolve_current_emotion(&self, previous: Option<&str>, analyzed: &EmotionResult) -> Emotion;
}

/// Detects in-turn events and supplies impact/confidence weights per [`EventType`].
///
/// ## When to implement
///
/// - **Who**: authors of custom event detection / impact weighting policies.
/// - **When**: when working with [`EventEstimator`](crate::EventEstimator) and needing to change event classification or the weight table.
///
/// ## When not to implement
///
/// - When relying entirely on the builtin `DefaultEventPolicy` + `BuiltinEventEstimator`.
pub trait EventPolicy: Send + Sync {
    /// Detects this turn's dialogue event type.
    ///
    /// # Errors
    ///
    /// Returns an error when the policy cannot classify the message into an [`Event`].
    ///
    /// # Panics
    ///
    /// Does not panic.
    fn detect(&self, text: &str, user_emotion: &Emotion, bot_emotion: &Emotion) -> Result<Event>;

    /// Returns the narrative impact weight for the given event type.
    ///
    /// # Errors
    ///
    /// None; this method does not return a `Result`.
    ///
    /// # Panics
    ///
    /// Does not panic.
    fn impact(&self, event_type: &EventType) -> f64;

    /// Returns the detection confidence for the given event type.
    ///
    /// # Errors
    ///
    /// None; this method does not return a `Result`.
    ///
    /// # Panics
    ///
    /// Does not panic.
    fn confidence(&self, event_type: &EventType) -> f32;
}

/// Decides what to persist as long-term memory and with what importance.
///
/// ## When to implement
///
/// - **Who**: authors of policies that customize "which content is written to long-term memory and its importance scoring".
/// - **When**: when memory persistence rules need to change per role/scene.
///
/// ## When not to implement
///
/// - When the default `DefaultMemoryPolicy` already satisfies the role design.
pub trait MemoryPolicy: Send + Sync {
    /// Builds the memory body to persist based on the policy context.
    ///
    /// # Errors
    ///
    /// None; this method does not return a `Result`.
    ///
    /// # Panics
    ///
    /// Does not panic.
    fn build_memory_entry(&self, ctx: &PolicyContext<'_>) -> String;

    /// Decides whether this turn should be written to long-term memory.
    ///
    /// # Errors
    ///
    /// None; this method does not return a `Result`.
    ///
    /// # Panics
    ///
    /// Does not panic.
    fn should_persist(&self, ctx: &PolicyContext<'_>) -> bool;

    /// Computes the importance score of this turn's memory.
    ///
    /// # Errors
    ///
    /// None; this method does not return a `Result`.
    ///
    /// # Panics
    ///
    /// Does not panic.
    fn importance(&self, ctx: &PolicyContext<'_>) -> f64;

    /// Returns the FIFO eviction limit (number of entries).
    ///
    /// # Errors
    ///
    /// None; this method does not return a `Result`.
    ///
    /// # Panics
    ///
    /// Does not panic.
    fn fifo_limit(&self) -> i32;
}

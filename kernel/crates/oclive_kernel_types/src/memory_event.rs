//! Typed memory events carried by the generic Event Ring.

use serde::{Deserialize, Serialize};

/// A memory module has found one recollection candidate for the current cue.
pub const MEMORY_RECALL_CANDIDATE_EVENT_KIND: &str = "kernel.memory.recall.candidate";

/// A decision module has admitted one recollection into the current role context.
pub const MEMORY_RECOLLECTION_ACTIVATED_EVENT_KIND: &str = "kernel.memory.recollection.activated";

/// Why a memory became a recollection candidate.
///
/// # Examples
///
/// ```
/// use oclive_kernel_types::MemoryRecallReason;
///
/// assert_eq!(
///     serde_json::to_string(&MemoryRecallReason::CurrentMessageCue).unwrap(),
///     "\"current_message_cue\""
/// );
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MemoryRecallReason {
    CurrentMessageCue,
}

/// How an admitted recollection should affect a long-text reply.
///
/// # Examples
///
/// ```
/// use oclive_kernel_types::MemoryRecollectionExpressionMode;
///
/// let mode = MemoryRecollectionExpressionMode::Weave;
/// assert_eq!(serde_json::to_string(&mode).unwrap(), "\"weave\"");
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MemoryRecollectionExpressionMode {
    Latent,
    Weave,
    Explicit,
}

/// Memory-domain proposal emitted after retrieval finds a cue-related memory.
///
/// Scores use the Event Ring fixed-point scale (`10_000 == 1.0`). The event carries a memory
/// reference rather than full private memory text.
///
/// # Examples
///
/// ```
/// use oclive_kernel_types::{MemoryRecallCandidate, MemoryRecallReason};
///
/// let candidate = MemoryRecallCandidate {
///     memory_id: "memory-1".into(),
///     confidence_bps: 8_000,
///     salience_bps: 7_000,
///     reason: MemoryRecallReason::CurrentMessageCue,
/// };
/// assert_eq!(candidate.memory_id, "memory-1");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryRecallCandidate {
    pub memory_id: String,
    pub confidence_bps: u16,
    pub salience_bps: u16,
    pub reason: MemoryRecallReason,
}

/// Decision output admitting one memory into the current role context.
///
/// # Examples
///
/// ```
/// use oclive_kernel_types::{
///     MemoryRecollectionActivated, MemoryRecollectionExpressionMode,
/// };
///
/// let activated = MemoryRecollectionActivated {
///     memory_id: "memory-1".into(),
///     reply_influence_bps: 7_600,
///     expression_mode: MemoryRecollectionExpressionMode::Weave,
///     ttl_turns: 1,
/// };
/// assert_eq!(activated.ttl_turns, 1);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryRecollectionActivated {
    pub memory_id: String,
    pub reply_influence_bps: u16,
    pub expression_mode: MemoryRecollectionExpressionMode,
    pub ttl_turns: u8,
}

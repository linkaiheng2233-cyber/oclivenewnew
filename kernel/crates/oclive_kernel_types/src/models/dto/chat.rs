//! Chat invoke / response DTOs shared by universal clients.

use serde::{Deserialize, Serialize};

use super::AdultBeatDto;
use super::AdultInteractionRequest;

/// Primary chat invoke payload (`send_message`).
#[derive(Debug, Default, Deserialize)]
pub struct SendMessageRequest {
    pub role_id: String,
    pub user_message: String,
    #[serde(default)]
    pub scene_id: Option<String>,
    /// Optional: distinguishes multiple sessions for the same role (e.g. HTTP trial chat "new session"); combined with `role_id` as the internal DB namespace.
    #[serde(default)]
    pub session_id: Option<String>,
    /// When `true`, response may include `raw_reply` if post-processor changed the LLM text.
    #[serde(default)]
    pub include_raw_reply: Option<bool>,
    /// Optional Chat Pro-only adult interaction context. Omitted by universal clients.
    #[serde(default)]
    pub adult: Option<AdultInteractionRequest>,
}

/// UI-only affect metrics (simulation values; must not drive PromptBuilder mechanics).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DisplayMetricsDto {
    /// Favor score 0–100.
    pub favor: f64,
    /// Relation stage label (e.g. Stranger / Friend).
    pub relation_summary: String,
    /// Seven personality dimensions (stubbornness … warmth).
    pub traits: Vec<f64>,
}

/// Seven-dimensional emotion snapshot returned to the UI.
#[derive(Debug, Serialize, Deserialize)]
pub struct EmotionDto {
    pub joy: f32,
    pub sadness: f32,
    pub anger: f32,
    pub fear: f32,
    pub surprise: f32,
    pub disgust: f32,
    pub neutral: f32,
}

/// Serialized detected event for the chat response payload.
#[derive(Debug, Serialize, Deserialize)]
pub struct DetectedEventDto {
    pub event_type: String,
    pub confidence: f32,
}

/// `send_message` co-present / remote-presence stub / remote inner-voice modes (for UI styling and debugging).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PresenceMode {
    CoPresent,
    RemoteStub,
    RemoteLife,
}

/// Primary chat invoke result (`send_message`); field `reply` is the assistant text.
#[derive(Debug, Serialize, Deserialize)]
pub struct SendMessageResponse {
    pub api_version: u32,
    pub schema: u32,
    /// Co-present / remote-presence stub / remote inner voice.
    pub presence_mode: PresenceMode,
    /// UI-only affect snapshot (favor / traits / relation stage); simulation still runs in kernel.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_metrics: Option<DisplayMetricsDto>,
    /// Relation state after this turn (`role_runtime.relation_state`).
    #[deprecated(note = "use display_metrics.relation_summary")]
    pub relation_state: String,
    pub reply: String,
    /// Structured adult beat. `reply` remains the dialogue-only compatibility field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adult_beat: Option<AdultBeatDto>,
    /// User-input emotion analysis (seven dimensions); for debugging or advanced UI display.
    pub emotion: EmotionDto,
    /// Bot emotion label parsed this turn (lowercase English; matches `Emotion::Display`).
    pub bot_emotion: String,
    /// Portrait expression (LLM + persona + events combined; matches `role_runtime.current_emotion`).
    pub portrait_emotion: String,
    /// Closed-set catalog asset id when `portrait_catalog.enabled`; legacy packs omit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visual_state_id: Option<String>,
    /// Render directive from visual presentation facility (#4); omitted when disabled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub performance_directive:
        Option<super::super::visual_presentation_config::PerformanceDirective>,
    pub favorability_delta: f32,
    #[deprecated(note = "use display_metrics.favor")]
    pub favorability_current: f32,
    pub events: Vec<DetectedEventDto>,
    pub scene_id: String,
    /// Set by the backend when the user expresses travel/move intent; actual scene switch only via `switch_scene`.
    pub offer_destination_picker: bool,
    /// Set when rules/model detect the user inviting the role to travel together; confirm via `switch_scene` (`together: true`).
    #[serde(default)]
    pub offer_together_travel: bool,
    /// Whether a fallback short reply was used after main dialogue LLM failure (co-present or remote inner voice).
    #[serde(default)]
    pub reply_is_fallback: bool,
    /// When `reply_is_fallback = true`, optional main LLM failure reason (for UI hint; no full prompt).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub llm_fallback_reason: Option<String>,
    /// Knowledge chunks retrieved and injected into main/remote prompt this turn (0 = none injected or no hit).
    #[serde(default)]
    pub knowledge_chunks_in_prompt: u32,
    pub timestamp: i64,
    /// User row id after CoPresent writes to `chat_messages`; `None` if not written.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_message_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assistant_message_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_message_timestamp: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assistant_message_timestamp: Option<String>,
    /// `true` when CoPresent chat row persistence failed (SQLite authoritative store).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_persist_failed: Option<bool>,
    /// Human-readable chat persistence error when `chat_persist_failed` is set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_persist_error: Option<String>,
    /// `true` when role blueprint enables dual-core but host fell back to co-present path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dual_core_degraded: Option<bool>,
    /// Pre–post-processor LLM text; only when `include_raw_reply` was requested and text changed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_reply: Option<String>,
    /// Ollama `prompt_eval_duration` (ms) when `OCLIVE_BENCH_TELEMETRY=1` and Deep prefix-cache path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub llm_prompt_eval_ms: Option<u64>,
}

// ----- WEEK3-004: role / memory / event queries -----

/// Sentinel submitted for the identity dropdown option "follow creator manifest default identity" (not a manifest key).
pub const OCLIVE_DEFAULT_RELATION_SENTINEL: &str = "__oclive_default__";

/// Sentinel for User Identity Prompt Template picker "follow pack default" (same value as relation sentinel).
pub const OCLIVE_DEFAULT_IDENTITY_SENTINEL: &str = "__oclive_default__";

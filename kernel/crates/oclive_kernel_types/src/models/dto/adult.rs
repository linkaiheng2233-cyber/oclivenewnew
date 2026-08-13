//! Adult interaction DTOs (Chat Pro adult extension).

use serde::{Deserialize, Serialize};

use super::SendMessageResponse;

/// Why an adult-capable turn is being generated.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdultInteractionAction {
    /// An ordinary visible user turn.
    #[default]
    Message,
    /// Continue the current interaction without inventing words for the user.
    Continue,
    /// User pressed the explicit exit button.
    Exit,
}

/// Chat Pro gate and per-session state supplied to the kernel for this turn.
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct AdultInteractionRequest {
    #[serde(default)]
    pub confirmed_adult: bool,
    #[serde(default)]
    pub global_enabled: bool,
    #[serde(default)]
    pub role_enabled: bool,
    #[serde(default)]
    pub interaction_active: bool,
    #[serde(default)]
    pub action: AdultInteractionAction,
    /// Present only for a background-generated beat. The kernel generates the
    /// reply without making it visible or committing turn side effects.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage: Option<AdultStageDirective>,
}

impl AdultInteractionRequest {
    #[must_use]
    pub fn gates_open(&self) -> bool {
        self.confirmed_adult && self.global_enabled && self.role_enabled
    }
}

/// Identifies one ordered beat inside a cancellable background generation.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct AdultStageDirective {
    pub generation_id: String,
    pub sequence: u32,
}

/// Model-declared state after a structured adult-capable reply.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdultInteractionState {
    #[default]
    Inactive,
    Active,
    Ended,
}

/// Structured role dialogue + silent narration returned to Chat Pro.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AdultBeatDto {
    pub dialogue: String,
    pub narration: String,
    pub interaction_state: AdultInteractionState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_beat_interval_ms: Option<u64>,
}

/// Start a new staged generation for one role/session/scene. Starting a new
/// generation invalidates any still-pending generation for the same chat.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BeginAdultStageGenerationRequest {
    pub role_id: String,
    #[serde(default)]
    pub scene_id: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
    pub adult: AdultInteractionRequest,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BeginAdultStageGenerationResponse {
    pub generation_id: String,
    pub next_sequence: u32,
}

/// Generate and durably stage one continuation beat.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StageAdultBeatRequest {
    pub role_id: String,
    #[serde(default)]
    pub scene_id: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
    pub generation_id: String,
    pub sequence: u32,
    pub adult: AdultInteractionRequest,
}

/// A staged beat is not part of visible chat history until explicitly
/// committed by the foreground chat.
#[derive(Debug, Serialize, Deserialize)]
pub struct AdultStagedBeatDto {
    pub generation_id: String,
    pub sequence: u32,
    pub response: SendMessageResponse,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CommitAdultStagedBeatRequest {
    pub role_id: String,
    #[serde(default)]
    pub scene_id: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
    pub generation_id: String,
    pub sequence: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CancelAdultStageGenerationRequest {
    pub role_id: String,
    #[serde(default)]
    pub scene_id: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
    pub generation_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListAdultStagedBeatsRequest {
    pub role_id: String,
    #[serde(default)]
    pub scene_id: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
    pub generation_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListAdultStagedBeatsResponse {
    pub generation_id: String,
    pub active: bool,
    pub next_sequence: u32,
    pub beats: Vec<AdultStagedBeatDto>,
}

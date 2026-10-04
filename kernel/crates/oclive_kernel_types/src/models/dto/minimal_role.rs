//! Additive reference Host text interaction for prepared minimal roles.
//!
//! These types do not replace the rich chat wire DTOs or promise persistence,
//! recovery, streaming, rendering or persisted memory. The optional conversation
//! carries only caller-supplied completed turns of the current temporary binding.

use serde::{Deserialize, Serialize};

/// One text invocation. The prepared Host handle supplies the technical role ID.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MinimalRoleMessageRequest {
    pub user_message: String,
    /// Additional commitments are explicit; the current minimal Prompt rejects
    /// any nonempty value rather than silently dropping it.
    #[serde(default)]
    pub requirements: String,
}

/// An explicit converter-produced local source for the reference Host.
/// This is transport metadata, not a universal package layout or extra author
/// content. The Host owns byte budgets; the definition reference stays relative
/// to the absolute asset root. No implicit filename or rich-pack fallback exists.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MinimalRoleLocalSource {
    pub role_id: String,
    pub asset_root: String,
    pub definition_reference: String,
}

/// Caller-supplied quoted material from one completed temporary conversation turn.
/// This is not a storage identity, receipt or proof that a server committed it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MinimalRoleConversationTurn {
    pub user_message: String,
    pub reply: String,
}

/// One non-streaming, non-persistent reference Host invocation from a local
/// converter source. The transport binds the default Literal Prompt and rejects
/// nonempty requirements instead of silently discarding them.
///
/// ```
/// use oclive_kernel_types::models::dto::MinimalRoleLocalMessageRequest;
/// let request: MinimalRoleLocalMessageRequest = serde_json::from_str(r#"{
///   "source": {"role_id":"local-id", "asset_root":"/installed/content",
///     "definition_reference":"chosen-content.json"},
///   "message": {"user_message":"hello"}
/// }"#).unwrap();
/// assert!(request.message.requirements.is_empty());
/// let with_conversation: oclive_kernel_types::models::dto::MinimalRoleLocalConversationRequest
///     = request.clone().into();
/// assert!(with_conversation.conversation.is_empty());
/// assert_eq!(request.source.role_id, "local-id");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MinimalRoleLocalMessageRequest {
    pub source: MinimalRoleLocalSource,
    pub message: MinimalRoleMessageRequest,
}

/// Additive transport envelope for quoted current-binding conversation.
/// The original local request stays source compatible, including struct literals.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MinimalRoleLocalConversationRequest {
    pub source: MinimalRoleLocalSource,
    pub message: MinimalRoleMessageRequest,
    /// Optional current-binding candidates, oldest first. The reference Host
    /// accepts at most eight complete turns and 64 KiB of combined UTF-8 content.
    /// Missing means empty; explicit null is not a conversation. No rich memory
    /// or saved history is opened by this field.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conversation: Vec<MinimalRoleConversationTurn>,
}

impl From<MinimalRoleLocalMessageRequest> for MinimalRoleLocalConversationRequest {
    fn from(request: MinimalRoleLocalMessageRequest) -> Self {
        Self {
            source: request.source,
            message: request.message,
            conversation: Vec::new(),
        }
    }
}

/// Product extensions are not executed by the basic text path. This does not
/// report the availability of other Host paths or the Kernel's Base slots.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MinimalRoleProductExtensionStatus {
    Unavailable,
}

/// A completed text call, without synthetic relationship/personality/emotion data.
/// A normally completed model call may return empty text. No chat row IDs or
/// idempotent outcome are implied by this result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MinimalRoleMessageResponse {
    pub role_id: String,
    pub reply: String,
    pub product_extensions: MinimalRoleProductExtensionStatus,
}

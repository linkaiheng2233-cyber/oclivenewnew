//! Additive, in-process reference Host text interaction for prepared minimal roles.
//!
//! These types do not replace the rich chat wire DTOs or promise persistence,
//! recovery, streaming, rendering or multi-turn memory.

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

//! User identity and relation DTOs.

use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

/// One entry from `user_identities/index.json` (`get_user_identity_state`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserIdentityDto {
    pub id: String,
    pub display_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maps_to_relation_id: Option<String>,
    /// Legacy/advisory role-pack metadata; not an adult runtime gate.
    #[serde(default = "default_true")]
    pub adult_eligible: bool,
}

/// Switch active User Identity Prompt Template (`set_user_identity`).
#[derive(Debug, Clone, Deserialize)]
pub struct SetUserIdentityRequest {
    pub role_id: String,
    pub identity_id: String,
}

/// Per-scene identity override when `identity_binding = per_scene`.
#[derive(Debug, Clone, Deserialize)]
pub struct SetSceneUserIdentityRequest {
    pub role_id: String,
    pub scene_id: String,
    pub identity_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetUserIdentityStateRequest {
    pub role_id: String,
    #[serde(default)]
    pub scene_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserIdentityStateResponse {
    pub role_id: String,
    pub identities: Vec<UserIdentityDto>,
    pub default_identity_id: String,
    pub current_identity_id: String,
    pub use_manifest_default: bool,
    pub effective_relation_key: String,
}

/// Manifest-defined user identity option (`get_role_info` / relation pickers).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRelationDto {
    pub id: String,
    pub name: String,
    pub prompt_hint: String,
    pub favor_multiplier: f32,
    /// Initial favorability configured for this identity in the role pack (0–100); synced to current favorability on identity switch.
    pub initial_favorability: f64,
}

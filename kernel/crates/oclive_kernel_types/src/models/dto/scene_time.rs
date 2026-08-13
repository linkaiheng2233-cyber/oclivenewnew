//! Time state, scene switch, and presence DTOs.

use serde::{Deserialize, Serialize};

use super::RoleInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeStateResponse {
    pub virtual_time_ms: i64,
    pub iso_datetime: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetRoleInteractionModeRequest {
    pub role_id: String,
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JumpTimeRequest {
    pub role_id: String,
    #[serde(default)]
    pub timestamp_ms: Option<i64>,
    #[serde(default)]
    pub preset: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JumpTimeResponse {
    pub virtual_time_ms: i64,
    pub iso_datetime: String,
    /// Monologues generated after time jump (typically 2; for frontend chat insertion).
    pub monologues: Vec<String>,
    pub favorability_delta: f32,
    pub favorability_current: f32,
    /// When `autonomous_scene` rule switches role `current_scene` from `from` to `to`.
    #[serde(default)]
    pub autonomous_scene_from: Option<String>,
    #[serde(default)]
    pub autonomous_scene_to: Option<String>,
}

fn default_switch_together() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchSceneRequest {
    pub role_id: String,
    pub scene_id: String,
    /// `true`: write `current_scene` and treat as co-present with role; `false`: only update `user_presence_scene` (narrative solitude).
    #[serde(default = "default_switch_together")]
    pub together: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetUserPresenceSceneRequest {
    pub role_id: String,
    pub scene_id: String,
}

/// Pre-import preview for `.ocpak` (manifest).
#[derive(Debug, Clone, Serialize)]
pub struct RolePackPeekResponse {
    pub id: String,
    pub name: String,
    pub version: String,
}

/// `switch_scene` response: role info and scene welcome message (for frontend chat insertion).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchSceneResponse {
    #[serde(flatten)]
    pub role: RoleInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scene_welcome: Option<String>,
}

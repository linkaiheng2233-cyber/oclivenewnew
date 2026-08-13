//! Monologue and chat-log export DTOs.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct GenerateMonologueRequest {
    pub role_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GenerateMonologueResponse {
    pub text: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExportChatLogsRequest {
    pub role_id: Option<String>,
    #[serde(default)]
    pub all_roles: bool,
    pub format: String,
    /// Optional: attach plugin backend resolution diagnostics to export (default off; ignored when `all_roles=true`).
    #[serde(default)]
    pub include_plugin_resolution_debug: bool,
    /// Optional session id for diagnostic namespace (only when `include_plugin_resolution_debug=true` and single-role export).
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExportChatLogsResponse {
    pub content: String,
    pub suggested_filename: String,
}

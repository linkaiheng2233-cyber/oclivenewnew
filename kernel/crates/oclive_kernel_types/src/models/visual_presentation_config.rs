//! Role pack `config.json` → `visual_presentation` (facility #4; default off).

use serde::{Deserialize, Serialize};

/// `config.json` → `visual_presentation`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct RolePackVisualPresentationConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_visual_backend")]
    pub backend: VisualPresentationBackendKind,
}

fn default_visual_backend() -> VisualPresentationBackendKind {
    VisualPresentationBackendKind::Image
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum VisualPresentationBackendKind {
    #[default]
    Image,
    Live2d,
    Rig3d,
    Procedural,
    Directory,
}

/// Host UI render directive (optional on `SendMessageResponse`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PerformanceDirective {
    pub visual_state_id: String,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expression: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub motion: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback_image: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub live2d_model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rig3d_model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

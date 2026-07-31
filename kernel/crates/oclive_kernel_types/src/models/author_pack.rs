//! Role pack `author.json`: creator notes, recommended plugins, and UI/backend suggestions (does not replace `pipeline.ocblueprint`).

use super::plugin_backends::PluginBackends;
use super::ui_config::UiConfig;
use serde::{Deserialize, Serialize};

/// Recommended directory plugin to install (market / version semantics consumed by the frontend and future validation).
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct AuthorRecommendedPlugin {
    pub id: String,
    /// e.g. `^1.0.0`; optional
    #[serde(default)]
    pub version_range: Option<String>,
    #[serde(default)]
    pub slots: Vec<String>,
    /// Backend modules this suggestion applies to (e.g. `memory`); optional
    #[serde(default)]
    pub for_backends: Vec<String>,
    #[serde(default)]
    pub optional: bool,
    #[serde(default)]
    pub note: Option<String>,
}

/// Top-level on-disk `author.json` (alongside other role pack files under `roles/{id}/`).
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct AuthorPackFile {
    #[serde(default = "author_pack_schema_v1")]
    pub schema_version: u32,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub detail_markdown: String,
    #[serde(default)]
    pub recommended_plugins: Vec<AuthorRecommendedPlugin>,
    /// Same shape as `ui.json`; when non-empty, takes precedence over `ui.json` as plugin UI seed/reset baseline.
    #[serde(default)]
    pub suggested_ui: Option<UiConfig>,
    /// Same shape as `settings.json` → `plugin_backends`; suggestion only—applied after UI confirmation via session override, etc.
    #[serde(default)]
    pub suggested_plugin_backends: Option<PluginBackends>,
}

fn author_pack_schema_v1() -> u32 {
    1
}

impl AuthorPackFile {
    /// Parse from UTF-8 JSON; returns `None` on failure (caller may log).
    #[must_use]
    pub fn from_json_str(raw: &str) -> Option<Self> {
        serde_json::from_str(raw.trim()).ok()
    }
}

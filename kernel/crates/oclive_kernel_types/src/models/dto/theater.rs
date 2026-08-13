//! Theater (AI theater) DTOs.

use serde::{Deserialize, Serialize};

/// Theater cast member reference (`generate_theater_scene`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TheaterCastRef {
    pub role_id: String,
    pub name: String,
}

/// One scripted beat in a theater scene.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TheaterScriptLine {
    pub id: String,
    pub cast: String,
    pub name: String,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage_hint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emotion: Option<String>,
}

/// Poke chip brief for `cast_rewrite` (chip_id + drama intent).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TheaterPokeChipDef {
    pub chip_id: String,
    pub drama_seed: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// Fork patch template for cast adaptation (`mode = cast_adapt`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TheaterForkTemplate {
    pub chip_id: String,
    pub insert_after_beat_id: String,
    pub patch_lines: Vec<TheaterScriptLine>,
}

/// User-applied poke / custom tweak metadata for scene director rewrite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TheaterTweak {
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chip_label: Option<String>,
    pub drama_seed: String,
    pub insert_after_beat_id: String,
    pub lead_cast: String,
}

/// `generate_theater_scene` request — full-scene structured rewrite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TheaterSceneRequest {
    pub cast_a: TheaterCastRef,
    pub cast_b: TheaterCastRef,
    pub scene_id: String,
    pub base_beats: Vec<TheaterScriptLine>,
    pub applied_tweaks: Vec<TheaterTweak>,
    pub fallback_beats: Vec<TheaterScriptLine>,
    #[serde(default)]
    pub max_beats: Option<u32>,
    /// `cast_adapt` | `cast_rewrite` | `ripple` (JSON ripple rewrite) | `patch` (local prose micro-scene).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    /// Patch mode only: `0` = first variant, `1` = alternate plot branch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub patch_variant: Option<u8>,
    /// Fork patch templates (name-bound baseline) for `cast_adapt`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fork_templates: Option<Vec<TheaterForkTemplate>>,
    /// Cast-adapt pass: `voice` | `depth` | `polish` (multi-round persona rewrite).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adapt_pass: Option<String>,
    /// Poke chip definitions for `cast_rewrite`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub poke_chips: Option<Vec<TheaterPokeChipDef>>,
    /// Pair-relation preset id (`family` | `friend` | `stranger` | `lover`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pair_relation_id: Option<String>,
    /// Human-readable pair-relation tone for cast_rewrite / ripple prompts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pair_relation_hint: Option<String>,
    /// Theater scene preset id (`breakfast` | `supermarket` | …); orthogonal to `scene_id`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theater_scene: Option<String>,
    /// Short scene description for cast_rewrite / ripple prompts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scene_brief: Option<String>,
    /// Scene constraints (location, time, forbidden elements).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scene_setting_hint: Option<String>,
    /// Mode 2: user script outline for `outline_rewrite`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script_outline: Option<String>,
}

/// `generate_theater_scene` response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TheaterSceneResponse {
    pub beats: Vec<TheaterScriptLine>,
    /// `local` | `cloud` | `fallback`
    pub source: String,
    pub model: String,
    /// Adapted fork patches when `mode = cast_adapt`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adapted_forks: Option<Vec<TheaterForkTemplate>>,
    /// Machine-readable hint when `source = "fallback"` (e.g. `rewrite_llm_timeout`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_reason: Option<String>,
    /// Partial success note (e.g. `rewrite_forks_template` when beats OK but forks reused).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rewrite_note: Option<String>,
}

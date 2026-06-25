//! Theater Scene Director — prompt-only directory plugin port (independent channel, not six-slot).

use oclive_kernel_types::models::{
    TheaterForkTemplate, TheaterPokeChipDef, TheaterScriptLine, TheaterTweak,
};
use oclive_kernel_types::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Maximum accepted prompt length from `theater.build_prompt` (directory plugin).
pub const MAX_THEATER_PROMPT_LEN: usize = 32_768;

/// JSON-RPC method for directory theater director plugins.
pub const THEATER_BUILD_PROMPT_METHOD: &str = "theater.build_prompt";

/// Effective backend kind for theater prompt resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TheaterDirectorBackendKind {
    Builtin,
    Directory,
}

/// Merged host profile + env result for theater director wiring.
#[derive(Debug, Clone)]
pub struct TheaterDirectorEffectiveConfig {
    pub backend: TheaterDirectorBackendKind,
    pub directory_plugin_id: String,
}

/// Input for one `theater.build_prompt` call (JSON-RPC params projection).
///
/// Derives `Default` so host-side constructors can fill only the fields that
/// distinguish a mode and leave the rest at their zero value via
/// `..Default::default()` (avoids ~40-field repetition per mode).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TheaterPromptBuildInput {
    /// `patch` | `ripple` | `cast_adapt` | `cast_rewrite` | `cast_rewrite_minimal` | `outline_rewrite`
    pub mode: String,
    pub strict: bool,
    pub persona_a: String,
    pub persona_b: String,
    pub cast_a_name: String,
    pub cast_b_name: String,
    pub cast_a_role_id: String,
    pub cast_b_role_id: String,
    pub scene_id: String,
    pub applied_tweaks: Vec<TheaterTweak>,
    pub base_beats: Vec<TheaterScriptLine>,
    pub max_beats: u32,
    pub patch_variant: Option<u8>,
    pub fork_templates: Option<Vec<TheaterForkTemplate>>,
    pub adapt_pass: Option<String>,
    pub poke_chips: Option<Vec<TheaterPokeChipDef>>,
    pub pair_relation_id: Option<String>,
    pub pair_relation_hint: Option<String>,
    pub theater_scene: Option<String>,
    pub scene_brief: Option<String>,
    pub scene_setting_hint: Option<String>,
    /// Ripple: immutable prefix beats (JSON rewrite modes).
    pub ripple_prefix_beats: Option<Vec<TheaterScriptLine>>,
    pub ripple_skeleton: Option<Vec<TheaterScriptLine>>,
    pub ripple_full_rewrite: Option<bool>,
    /// Patch mode context.
    pub patch_prefix_beats: Option<Vec<TheaterScriptLine>>,
    pub patch_skeleton_tail: Option<Vec<TheaterScriptLine>>,
    pub patch_canned_patch: Option<Vec<TheaterScriptLine>>,
    pub patch_tweak: Option<TheaterTweak>,
    pub patch_chip_slug: Option<String>,
    pub patch_max_lines: Option<u32>,
    /// Cast rewrite bounds.
    pub cast_rewrite_min_beats: Option<u32>,
    pub cast_rewrite_max_beats: Option<u32>,
    pub cast_rewrite_target_beats: Option<u32>,
    /// Mode 2: user script outline (`outline_rewrite`).
    pub script_outline: Option<String>,
}

/// Output of [`TheaterDirectorPromptProvider::build_prompt`].
#[derive(Debug, Clone)]
pub struct TheaterPromptBuildOutput {
    pub prompt: String,
}

/// Theater director prompt provider (`builtin` or `directory`).
pub trait TheaterDirectorPromptProvider: Send + Sync {
    /// Build the LLM instruction prompt for the requested theater mode.
    ///
    /// # Errors
    ///
    /// Directory implementations should return `Err` on RPC/validation failure; host may fall back to builtin.
    fn build_prompt(&self, input: &TheaterPromptBuildInput) -> Result<TheaterPromptBuildOutput>;
}

/// Factory port: directory JSON-RPC wiring lives in infrastructure only.
pub trait TheaterDirectorResolver: Send + Sync {
    /// Resolve `backend=directory` (builtin fallback on misconfiguration or RPC failure at call site).
    fn resolve_directory(
        &self,
        eff: &TheaterDirectorEffectiveConfig,
    ) -> Arc<dyn TheaterDirectorPromptProvider>;
}

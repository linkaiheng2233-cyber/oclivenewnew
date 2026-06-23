//! Resolve [`TheaterDirectorPromptProvider`] from distro profile + env (independent of six-slot `PluginHost`).

use crate::domain::builtin_theater_director::BuiltinTheaterDirector;
use crate::domain::host_profile::ENV_THEATER_DIRECTOR_PLUGIN;
use crate::domain::theater::patch_scene::PatchContext;
use crate::domain::theater::scene_director::{RippleContext, cast_rewrite_target_beats};
use crate::models::dto::TheaterSceneRequest;
use crate::state::AppState;
use oclive_kernel_contracts::{
    TheaterDirectorBackendKind, TheaterDirectorEffectiveConfig, TheaterDirectorPromptProvider,
    TheaterPromptBuildInput,
};
use std::sync::Arc;

#[must_use]
pub fn resolve_effective_theater_director_config(state: &AppState) -> TheaterDirectorEffectiveConfig {
    let env_id = std::env::var(ENV_THEATER_DIRECTOR_PLUGIN)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let profile_id = state
        .host_profile
        .theater
        .director_plugin
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let plugin_id = env_id.or(profile_id).unwrap_or_default();
    if plugin_id.is_empty() {
        return TheaterDirectorEffectiveConfig {
            backend: TheaterDirectorBackendKind::Builtin,
            directory_plugin_id: String::new(),
        };
    }
    let rt = &state.directory_plugins;
    if rt.plugin_roots.read().contains_key(&plugin_id)
        && rt.manifest_provides_capability(&plugin_id, "theater_director")
    {
        return TheaterDirectorEffectiveConfig {
            backend: TheaterDirectorBackendKind::Directory,
            directory_plugin_id: plugin_id,
        };
    }
    tracing::warn!(
        target: "oclive_theater",
        plugin_id = %plugin_id,
        "theater director plugin missing or lacks provides theater_director; builtin fallback"
    );
    TheaterDirectorEffectiveConfig {
        backend: TheaterDirectorBackendKind::Builtin,
        directory_plugin_id: String::new(),
    }
}

#[must_use]
pub fn resolve_theater_director(state: &AppState) -> Arc<dyn TheaterDirectorPromptProvider> {
    let eff = resolve_effective_theater_director_config(state);
    match eff.backend {
        TheaterDirectorBackendKind::Builtin => Arc::new(BuiltinTheaterDirector),
        TheaterDirectorBackendKind::Directory => {
            state.theater_director_resolver.as_ref().resolve_directory(&eff)
        }
        _ => Arc::new(BuiltinTheaterDirector),
    }
}

/// Build prompt via resolver; on failure falls back to builtin templates.
#[must_use]
pub fn build_theater_prompt(state: &AppState, input: &TheaterPromptBuildInput) -> String {
    let provider = resolve_theater_director(state);
    match provider.build_prompt(input) {
        Ok(out) if !out.prompt.trim().is_empty() => out.prompt,
        Ok(_) => {
            tracing::warn!(
                target: "oclive_theater",
                mode = %input.mode,
                "theater director returned empty prompt; builtin fallback"
            );
            BuiltinTheaterDirector::build_prompt_inner(input)
        }
        Err(e) => {
            tracing::warn!(
                target: "oclive_theater",
                mode = %input.mode,
                error = %e,
                "theater director build_prompt failed; builtin fallback"
            );
            BuiltinTheaterDirector::build_prompt_inner(input)
        }
    }
}

#[must_use]
pub(crate) fn ripple_prompt_input(
    req: &TheaterSceneRequest,
    ctx: &RippleContext,
    max_beats: u32,
    strict: bool,
    persona_a: &str,
    persona_b: &str,
) -> TheaterPromptBuildInput {
    TheaterPromptBuildInput {
        mode: "ripple".to_string(),
        strict,
        persona_a: persona_a.to_string(),
        persona_b: persona_b.to_string(),
        cast_a_name: req.cast_a.name.clone(),
        cast_b_name: req.cast_b.name.clone(),
        cast_a_role_id: req.cast_a.role_id.clone(),
        cast_b_role_id: req.cast_b.role_id.clone(),
        scene_id: req.scene_id.clone(),
        applied_tweaks: req.applied_tweaks.clone(),
        base_beats: req.base_beats.clone(),
        max_beats,
        patch_variant: None,
        fork_templates: None,
        adapt_pass: None,
        poke_chips: req.poke_chips.clone(),
        pair_relation_id: req.pair_relation_id.clone(),
        pair_relation_hint: req.pair_relation_hint.clone(),
        theater_scene: req.theater_scene.clone(),
        scene_brief: req.scene_brief.clone(),
        scene_setting_hint: req.scene_setting_hint.clone(),
        ripple_prefix_beats: Some(ctx.prefix_beats.clone()),
        ripple_skeleton: Some(ctx.ripple_skeleton.clone()),
        ripple_full_rewrite: Some(ctx.full_rewrite),
        patch_prefix_beats: None,
        patch_skeleton_tail: None,
        patch_canned_patch: None,
        patch_tweak: None,
        patch_chip_slug: None,
        patch_max_lines: None,
        cast_rewrite_min_beats: None,
        cast_rewrite_max_beats: None,
        cast_rewrite_target_beats: None,
    }
}

#[must_use]
pub fn cast_adapt_prompt_input(
    req: &TheaterSceneRequest,
    max_beats: u32,
    strict: bool,
    persona_a: &str,
    persona_b: &str,
) -> TheaterPromptBuildInput {
    TheaterPromptBuildInput {
        mode: "cast_adapt".to_string(),
        strict,
        persona_a: persona_a.to_string(),
        persona_b: persona_b.to_string(),
        cast_a_name: req.cast_a.name.clone(),
        cast_b_name: req.cast_b.name.clone(),
        cast_a_role_id: req.cast_a.role_id.clone(),
        cast_b_role_id: req.cast_b.role_id.clone(),
        scene_id: req.scene_id.clone(),
        applied_tweaks: req.applied_tweaks.clone(),
        base_beats: req.base_beats.clone(),
        max_beats,
        patch_variant: None,
        fork_templates: req.fork_templates.clone(),
        adapt_pass: req.adapt_pass.clone(),
        poke_chips: req.poke_chips.clone(),
        pair_relation_id: req.pair_relation_id.clone(),
        pair_relation_hint: req.pair_relation_hint.clone(),
        theater_scene: req.theater_scene.clone(),
        scene_brief: req.scene_brief.clone(),
        scene_setting_hint: req.scene_setting_hint.clone(),
        ripple_prefix_beats: None,
        ripple_skeleton: None,
        ripple_full_rewrite: None,
        patch_prefix_beats: None,
        patch_skeleton_tail: None,
        patch_canned_patch: None,
        patch_tweak: None,
        patch_chip_slug: None,
        patch_max_lines: None,
        cast_rewrite_min_beats: None,
        cast_rewrite_max_beats: None,
        cast_rewrite_target_beats: None,
    }
}

#[must_use]
pub fn cast_rewrite_prompt_input(
    req: &TheaterSceneRequest,
    min_beats: u32,
    max_beats: u32,
    strict: bool,
    persona_a: &str,
    persona_b: &str,
) -> TheaterPromptBuildInput {
    TheaterPromptBuildInput {
        mode: "cast_rewrite".to_string(),
        strict,
        persona_a: persona_a.to_string(),
        persona_b: persona_b.to_string(),
        cast_a_name: req.cast_a.name.clone(),
        cast_b_name: req.cast_b.name.clone(),
        cast_a_role_id: req.cast_a.role_id.clone(),
        cast_b_role_id: req.cast_b.role_id.clone(),
        scene_id: req.scene_id.clone(),
        applied_tweaks: req.applied_tweaks.clone(),
        base_beats: req.base_beats.clone(),
        max_beats,
        patch_variant: None,
        fork_templates: req.fork_templates.clone(),
        adapt_pass: None,
        poke_chips: req.poke_chips.clone(),
        pair_relation_id: req.pair_relation_id.clone(),
        pair_relation_hint: req.pair_relation_hint.clone(),
        theater_scene: req.theater_scene.clone(),
        scene_brief: req.scene_brief.clone(),
        scene_setting_hint: req.scene_setting_hint.clone(),
        ripple_prefix_beats: None,
        ripple_skeleton: None,
        ripple_full_rewrite: None,
        patch_prefix_beats: None,
        patch_skeleton_tail: None,
        patch_canned_patch: None,
        patch_tweak: None,
        patch_chip_slug: None,
        patch_max_lines: None,
        cast_rewrite_min_beats: Some(min_beats),
        cast_rewrite_max_beats: Some(max_beats),
        cast_rewrite_target_beats: Some(cast_rewrite_target_beats(min_beats, max_beats)),
    }
}

#[must_use]
pub fn cast_rewrite_minimal_prompt_input(
    req: &TheaterSceneRequest,
    target_beats: u32,
    persona_a: &str,
    persona_b: &str,
) -> TheaterPromptBuildInput {
    TheaterPromptBuildInput {
        mode: "cast_rewrite_minimal".to_string(),
        strict: true,
        persona_a: persona_a.to_string(),
        persona_b: persona_b.to_string(),
        cast_a_name: req.cast_a.name.clone(),
        cast_b_name: req.cast_b.name.clone(),
        cast_a_role_id: req.cast_a.role_id.clone(),
        cast_b_role_id: req.cast_b.role_id.clone(),
        scene_id: req.scene_id.clone(),
        applied_tweaks: req.applied_tweaks.clone(),
        base_beats: req.base_beats.clone(),
        max_beats: target_beats,
        patch_variant: None,
        fork_templates: req.fork_templates.clone(),
        adapt_pass: None,
        poke_chips: req.poke_chips.clone(),
        pair_relation_id: req.pair_relation_id.clone(),
        pair_relation_hint: req.pair_relation_hint.clone(),
        theater_scene: req.theater_scene.clone(),
        scene_brief: req.scene_brief.clone(),
        scene_setting_hint: req.scene_setting_hint.clone(),
        ripple_prefix_beats: None,
        ripple_skeleton: None,
        ripple_full_rewrite: None,
        patch_prefix_beats: None,
        patch_skeleton_tail: None,
        patch_canned_patch: None,
        patch_tweak: None,
        patch_chip_slug: None,
        patch_max_lines: None,
        cast_rewrite_min_beats: None,
        cast_rewrite_max_beats: None,
        cast_rewrite_target_beats: Some(target_beats),
    }
}

#[must_use]
pub(crate) fn patch_prompt_input(
    req: &TheaterSceneRequest,
    ctx: &PatchContext,
    max_lines: usize,
    strict: bool,
    persona_a: &str,
    persona_b: &str,
    variant_index: u8,
) -> TheaterPromptBuildInput {
    TheaterPromptBuildInput {
        mode: "patch".to_string(),
        strict,
        persona_a: persona_a.to_string(),
        persona_b: persona_b.to_string(),
        cast_a_name: req.cast_a.name.clone(),
        cast_b_name: req.cast_b.name.clone(),
        cast_a_role_id: req.cast_a.role_id.clone(),
        cast_b_role_id: req.cast_b.role_id.clone(),
        scene_id: req.scene_id.clone(),
        applied_tweaks: req.applied_tweaks.clone(),
        base_beats: req.base_beats.clone(),
        max_beats: req.max_beats.unwrap_or(12),
        patch_variant: Some(variant_index),
        fork_templates: None,
        adapt_pass: None,
        poke_chips: None,
        pair_relation_id: req.pair_relation_id.clone(),
        pair_relation_hint: req.pair_relation_hint.clone(),
        theater_scene: req.theater_scene.clone(),
        scene_brief: req.scene_brief.clone(),
        scene_setting_hint: req.scene_setting_hint.clone(),
        ripple_prefix_beats: None,
        ripple_skeleton: None,
        ripple_full_rewrite: None,
        patch_prefix_beats: Some(ctx.prefix_beats.clone()),
        patch_skeleton_tail: Some(ctx.skeleton_tail.clone()),
        patch_canned_patch: Some(ctx.canned_patch.clone()),
        patch_tweak: Some(ctx.tweak.clone()),
        patch_chip_slug: Some(ctx.chip_slug.clone()),
        patch_max_lines: Some(max_lines as u32),
        cast_rewrite_min_beats: None,
        cast_rewrite_max_beats: None,
        cast_rewrite_target_beats: None,
    }
}

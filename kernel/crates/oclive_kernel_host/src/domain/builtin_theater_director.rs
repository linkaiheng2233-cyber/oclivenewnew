//! Builtin theater director prompts — delegates to kernel `build_*_prompt` templates.

use crate::domain::theater::patch_scene::{build_patch_prompt, PatchContext};
use crate::domain::theater::scene_director::{
    build_cast_adapt_prompt, build_cast_rewrite_minimal_prompt, build_cast_rewrite_prompt,
    build_scene_prompt, RippleContext,
};
use crate::error::Result;
use crate::models::dto::TheaterSceneRequest;
use oclive_kernel_contracts::theater_director::{
    TheaterDirectorPromptProvider, TheaterPromptBuildInput, TheaterPromptBuildOutput,
};

#[derive(Debug, Default)]
pub struct BuiltinTheaterDirector;

impl BuiltinTheaterDirector {
    #[must_use]
    pub fn build_prompt_inner(input: &TheaterPromptBuildInput) -> String {
        let req = theater_request_from_input(input);
        let mode = input.mode.trim();
        match mode {
            "patch" => {
                let ctx = patch_context_from_input(input);
                build_patch_prompt(
                    &req,
                    &ctx,
                    input.patch_max_lines.unwrap_or(3) as usize,
                    input.strict,
                    input.persona_a.as_str(),
                    input.persona_b.as_str(),
                    input.patch_variant.unwrap_or(0),
                )
            }
            "ripple" => {
                let ctx = ripple_context_from_input(input);
                build_scene_prompt(
                    &req,
                    &ctx,
                    input.max_beats,
                    input.strict,
                    input.persona_a.as_str(),
                    input.persona_b.as_str(),
                )
            }
            "cast_adapt" => {
                let templates = input.fork_templates.clone().unwrap_or_default();
                build_cast_adapt_prompt(
                    &req,
                    &templates,
                    input.max_beats,
                    input.strict,
                    input.persona_a.as_str(),
                    input.persona_b.as_str(),
                )
            }
            "cast_rewrite" => {
                let min = input.cast_rewrite_min_beats.unwrap_or(6);
                let max = input.cast_rewrite_max_beats.unwrap_or(input.max_beats);
                build_cast_rewrite_prompt(
                    &req,
                    min,
                    max,
                    input.strict,
                    input.persona_a.as_str(),
                    input.persona_b.as_str(),
                )
            }
            "cast_rewrite_minimal" => {
                let target = input.cast_rewrite_target_beats.unwrap_or(input.max_beats);
                build_cast_rewrite_minimal_prompt(
                    &req,
                    target,
                    input.persona_a.as_str(),
                    input.persona_b.as_str(),
                )
            }
            other => {
                tracing::warn!(
                    target: "oclive_theater",
                    mode = %other,
                    "unknown theater prompt mode; using ripple"
                );
                let ctx = ripple_context_from_input(input);
                build_scene_prompt(
                    &req,
                    &ctx,
                    input.max_beats,
                    input.strict,
                    input.persona_a.as_str(),
                    input.persona_b.as_str(),
                )
            }
        }
    }
}

impl TheaterDirectorPromptProvider for BuiltinTheaterDirector {
    fn build_prompt(&self, input: &TheaterPromptBuildInput) -> Result<TheaterPromptBuildOutput> {
        Ok(TheaterPromptBuildOutput {
            prompt: Self::build_prompt_inner(input),
        })
    }
}

fn theater_request_from_input(input: &TheaterPromptBuildInput) -> TheaterSceneRequest {
    TheaterSceneRequest {
        cast_a: crate::models::dto::TheaterCastRef {
            role_id: input.cast_a_role_id.clone(),
            name: input.cast_a_name.clone(),
        },
        cast_b: crate::models::dto::TheaterCastRef {
            role_id: input.cast_b_role_id.clone(),
            name: input.cast_b_name.clone(),
        },
        scene_id: input.scene_id.clone(),
        base_beats: input.base_beats.clone(),
        applied_tweaks: input.applied_tweaks.clone(),
        fallback_beats: input.base_beats.clone(),
        max_beats: Some(input.max_beats),
        mode: Some(input.mode.clone()),
        patch_variant: input.patch_variant,
        fork_templates: input.fork_templates.clone(),
        adapt_pass: input.adapt_pass.clone(),
        poke_chips: input.poke_chips.clone(),
        pair_relation_id: input.pair_relation_id.clone(),
        pair_relation_hint: input.pair_relation_hint.clone(),
        theater_scene: input.theater_scene.clone(),
        scene_brief: input.scene_brief.clone(),
        scene_setting_hint: input.scene_setting_hint.clone(),
    }
}

fn ripple_context_from_input(input: &TheaterPromptBuildInput) -> RippleContext {
    RippleContext {
        prefix_beats: input.ripple_prefix_beats.clone().unwrap_or_default(),
        ripple_skeleton: input.ripple_skeleton.clone().unwrap_or_default(),
        full_rewrite: input.ripple_full_rewrite.unwrap_or(false),
    }
}

fn patch_context_from_input(input: &TheaterPromptBuildInput) -> PatchContext {
    let empty_tweak = crate::models::dto::TheaterTweak {
        kind: String::new(),
        chip_label: None,
        drama_seed: String::new(),
        insert_after_beat_id: String::new(),
        lead_cast: String::new(),
    };
    PatchContext {
        prefix_beats: input.patch_prefix_beats.clone().unwrap_or_default(),
        skeleton_tail: input.patch_skeleton_tail.clone().unwrap_or_default(),
        canned_patch: input.patch_canned_patch.clone().unwrap_or_default(),
        tweak: input.patch_tweak.clone().unwrap_or(empty_tweak),
        chip_slug: input.patch_chip_slug.clone().unwrap_or_default(),
    }
}

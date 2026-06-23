//! Theater scene director: one LLM call rewrites the ripple zone (after tweaks) as structured JSON beats.
//!
//! Bypasses `process_message` and six-slot orchestration; uses
//! [`PluginHost::llm_for_plugin_backends`](crate::domain::plugin_host::PluginHost::llm_for_plugin_backends)
//! with the effective model resolved from cast roles (same path as Chat Pro model manager).

use crate::domain::effective_llm_model::resolve_effective_ollama_model;
use crate::domain::theater::drama_guardrails;
use crate::domain::ports::LlmClient;
use crate::domain::user_llm_env::apply_user_llm_env;
use crate::domain::theater::scene_director_config::{
    cast_rewrite_llm_timeout_secs, cast_rewrite_min_beats, ripple_max_beats, scene_llm_timeout_secs,
};
use crate::error::{AppError, Result};
use crate::models::dto::{
    TheaterForkTemplate, TheaterPokeChipDef, TheaterSceneRequest, TheaterSceneResponse,
    TheaterScriptLine,
};
use crate::state::AppState;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

const MAX_BEAT_TEXT_LEN: usize = 500;
const MAX_STAGE_HINT_LEN: usize = 120;

struct CastRewriteParseCtx {
    name_a: String,
    name_b: String,
    role_id_a: String,
    role_id_b: String,
}

impl CastRewriteParseCtx {
    fn from_req(req: &TheaterSceneRequest) -> Self {
        Self {
            name_a: req.cast_a.name.trim().to_string(),
            name_b: req.cast_b.name.trim().to_string(),
            role_id_a: req.cast_a.role_id.trim().to_string(),
            role_id_b: req.cast_b.role_id.trim().to_string(),
        }
    }
}

pub(crate) fn cast_rewrite_target_beats(min_beats: u32, max_beats: u32) -> u32 {
    ((min_beats + max_beats) / 2).clamp(min_beats, 8)
}

pub(crate) fn scene_response(
    beats: Vec<TheaterScriptLine>,
    source: &str,
    model: String,
    adapted_forks: Option<Vec<TheaterForkTemplate>>,
) -> TheaterSceneResponse {
    scene_response_with_meta(beats, source, model, adapted_forks, None, None)
}

pub(crate) fn scene_response_with_meta(
    beats: Vec<TheaterScriptLine>,
    source: &str,
    model: String,
    adapted_forks: Option<Vec<TheaterForkTemplate>>,
    failure_reason: Option<&str>,
    rewrite_note: Option<&str>,
) -> TheaterSceneResponse {
    TheaterSceneResponse {
        beats,
        source: source.to_string(),
        model,
        adapted_forks,
        failure_reason: failure_reason.map(str::to_string),
        rewrite_note: rewrite_note.map(str::to_string),
    }
}

/// Prefix (immutable) + ripple skeleton for scene director rewrite.
pub(crate) struct RippleContext {
    pub(crate) prefix_beats: Vec<TheaterScriptLine>,
    pub(crate) ripple_skeleton: Vec<TheaterScriptLine>,
    pub(crate) full_rewrite: bool,
}

/// Resolve theater scene mode; infer `cast_rewrite` when optional `mode` was dropped by an older kernel DTO.
/// Modes: `cast_rewrite` | `cast_adapt` | `patch` (prose micro-scene) | `ripple` (JSON rewrite, default).
#[cfg_attr(test, allow(dead_code))]
pub(crate) fn resolve_scene_mode(req: &TheaterSceneRequest) -> Option<&str> {
    if let Some(mode) = req.mode.as_deref().map(str::trim).filter(|m| !m.is_empty()) {
        return Some(mode);
    }
    if req.base_beats.is_empty()
        && !req.fallback_beats.is_empty()
        && req.poke_chips.as_ref().is_some_and(|c| !c.is_empty())
    {
        return Some("cast_rewrite");
    }
    None
}

/// Generate a theater scene via LLM ripple rewrite; on parse failure retries once, then falls back.
///
/// # Errors
///
/// Returns [`AppError`] when model resolution or DB reads fail (not when LLM output is invalid —
/// that path returns `source = "fallback"`).
pub async fn generate_scene(
    state: &AppState,
    req: &TheaterSceneRequest,
) -> Result<TheaterSceneResponse> {
    if let Err(e) = apply_user_llm_env(state).await {
        tracing::warn!(
            target: "oclive_theater",
            error = %e,
            "apply_user_llm_env failed; continuing with cached LLM env"
        );
    }

    if resolve_scene_mode(req) == Some("cast_rewrite") {
        if req.fallback_beats.is_empty() {
            return Err(AppError::InvalidParameter(
                "theater cast_rewrite fallback_beats must not be empty".to_string(),
            ));
        }
        return generate_cast_rewrite_scene(state, req).await;
    }

    if req.base_beats.is_empty() {
        let hint = if !req.fallback_beats.is_empty() {
            " — cast apply needs `mode=cast_rewrite`; restart the app/kernel so :8420 loads the current build"
        } else {
            ""
        };
        return Err(AppError::InvalidParameter(format!(
            "theater scene base_beats must not be empty{hint}"
        )));
    }
    if req.fallback_beats.is_empty() {
        return Err(AppError::InvalidParameter(
            "theater scene fallback_beats must not be empty".to_string(),
        ));
    }

    if req.mode.as_deref() == Some("cast_adapt") {
        return generate_cast_adapt_scene(state, req).await;
    }

    if req.mode.as_deref() == Some("patch") {
        return crate::domain::theater::patch_scene::generate_patch_scene(state, req).await;
    }

    let ctx = resolve_ripple_context(req);
    let max_beats = req
        .max_beats
        .unwrap_or_else(ripple_max_beats)
        .clamp(4, 64);
    let (llm, model) = match resolve_theater_llm(state, req).await {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(target: "oclive_theater", "resolve_theater_llm failed: {e}");
            return Ok(fallback_response(req, state.ollama_model.as_str(), "fallback"));
        }
    };
    let source = resolve_llm_source_label(state);
    let timeout = Duration::from_secs(scene_llm_timeout_secs());

    let persona_a = resolve_cast_persona(state, req.cast_a.role_id.as_str()).await;
    let persona_b = resolve_cast_persona(state, req.cast_b.role_id.as_str()).await;

    let prompt = crate::domain::theater_director::build_theater_prompt(
        state,
        &crate::domain::theater_director::ripple_prompt_input(
            req,
            &ctx,
            max_beats,
            false,
            persona_a.as_str(),
            persona_b.as_str(),
        ),
    );
    let raw = match tokio::time::timeout(timeout, llm.generate_tag(model.as_str(), prompt.as_str())).await
    {
        Ok(Ok(t)) => t,
        Ok(Err(e)) => {
            tracing::warn!(target: "oclive_theater", "scene director LLM failed: {e}");
            return Ok(fallback_response(req, &model, source));
        }
        Err(_) => {
            tracing::warn!(target: "oclive_theater", "scene director LLM timed out ({}s)", scene_llm_timeout_secs());
            return Ok(fallback_response(req, &model, "fallback"));
        }
    };

    if let Some(beats) = try_merge_ripple(&ctx, &raw, max_beats) {
        return Ok(scene_response(beats, source, model.clone(), None));
    }

    let strict_prompt = crate::domain::theater_director::build_theater_prompt(
        state,
        &crate::domain::theater_director::ripple_prompt_input(
            req,
            &ctx,
            max_beats,
            true,
            persona_a.as_str(),
            persona_b.as_str(),
        ),
    );
    let retry_raw = match tokio::time::timeout(
        timeout,
        llm.generate_tag(model.as_str(), strict_prompt.as_str()),
    )
    .await
    {
        Ok(Ok(t)) => t,
        Ok(Err(e)) => {
            tracing::warn!(target: "oclive_theater", "scene director retry LLM failed: {e}");
            return Ok(fallback_response(req, &model, "fallback"));
        }
        Err(_) => {
            tracing::warn!(target: "oclive_theater", "scene director retry LLM timed out");
            return Ok(fallback_response(req, &model, "fallback"));
        }
    };

    if let Some(beats) = try_merge_ripple(&ctx, &retry_raw, max_beats) {
        return Ok(scene_response(beats, source, model, None));
    }

    tracing::warn!(target: "oclive_theater", "scene director parse failed; using fallback beats");
    Ok(fallback_response(req, &model, "fallback"))
}

fn fallback_response(
    req: &TheaterSceneRequest,
    model: &str,
    source: &str,
) -> TheaterSceneResponse {
    scene_response_with_meta(
        req.fallback_beats.clone(),
        source,
        model.to_string(),
        None,
        Some("ripple_parse_failed"),
        None,
    )
}

/// Cast-adapt mode: rewrite opening beats + fork patch lines in persona voice; ids immutable.
async fn generate_cast_adapt_scene(
    state: &AppState,
    req: &TheaterSceneRequest,
) -> Result<TheaterSceneResponse> {
    let max_beats = req
        .max_beats
        .unwrap_or_else(|| {
            (req.base_beats.len() as u32).max(ripple_max_beats())
        })
        .clamp(4, 64);
    let (llm, model) = match resolve_theater_llm(state, req).await {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(target: "oclive_theater", "cast_adapt resolve_theater_llm failed: {e}");
            return Ok(cast_adapt_fallback_response(req, state.ollama_model.as_str(), "fallback"));
        }
    };
    let source = resolve_llm_source_label(state);
    let timeout = Duration::from_secs(scene_llm_timeout_secs());
    let templates = req.fork_templates.clone().unwrap_or_default();

    let persona_a = resolve_cast_persona(state, req.cast_a.role_id.as_str()).await;
    let persona_b = resolve_cast_persona(state, req.cast_b.role_id.as_str()).await;

    let prompt = crate::domain::theater_director::build_theater_prompt(
        state,
        &crate::domain::theater_director::cast_adapt_prompt_input(
            req,
            max_beats,
            false,
            persona_a.as_str(),
            persona_b.as_str(),
        ),
    );
    let raw = match tokio::time::timeout(timeout, llm.generate_tag(model.as_str(), prompt.as_str())).await
    {
        Ok(Ok(t)) => t,
        Ok(Err(e)) => {
            tracing::warn!(target: "oclive_theater", "cast_adapt LLM failed: {e}");
            return Ok(cast_adapt_fallback_response(req, &model, "fallback"));
        }
        Err(_) => {
            tracing::warn!(target: "oclive_theater", "cast_adapt LLM timed out ({}s)", scene_llm_timeout_secs());
            return Ok(cast_adapt_fallback_response(req, &model, "fallback"));
        }
    };

    if let Some((beats, forks)) = try_merge_cast_adapt(req, &templates, &raw, max_beats) {
        return Ok(scene_response(beats, source, model.clone(), Some(forks)));
    }

    let strict_prompt = crate::domain::theater_director::build_theater_prompt(
        state,
        &crate::domain::theater_director::cast_adapt_prompt_input(
            req,
            max_beats,
            true,
            persona_a.as_str(),
            persona_b.as_str(),
        ),
    );
    let retry_raw = match tokio::time::timeout(
        timeout,
        llm.generate_tag(model.as_str(), strict_prompt.as_str()),
    )
    .await
    {
        Ok(Ok(t)) => t,
        Ok(Err(e)) => {
            tracing::warn!(target: "oclive_theater", "cast_adapt retry LLM failed: {e}");
            return Ok(cast_adapt_fallback_response(req, &model, "fallback"));
        }
        Err(_) => {
            tracing::warn!(target: "oclive_theater", "cast_adapt retry LLM timed out");
            return Ok(cast_adapt_fallback_response(req, &model, "fallback"));
        }
    };

    if let Some((beats, forks)) = try_merge_cast_adapt(req, &templates, &retry_raw, max_beats) {
        return Ok(scene_response(beats, source, model, Some(forks)));
    }

    tracing::warn!(target: "oclive_theater", "cast_adapt parse failed; using fallback");
    Ok(cast_adapt_fallback_response(req, &model, "fallback"))
}

/// Cast-rewrite mode: write a fresh breakfast script + poke forks from role personas (no skeleton merge).
async fn generate_cast_rewrite_scene(
    state: &AppState,
    req: &TheaterSceneRequest,
) -> Result<TheaterSceneResponse> {
    let fallback_len = req.fallback_beats.len().max(1) as u32;
    let min_beats = cast_rewrite_min_beats();
    let max_beats = req
        .max_beats
        .unwrap_or(fallback_len.max(10))
        .clamp(min_beats, 16);
    let (llm, model) = match resolve_theater_llm(state, req).await {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(target: "oclive_theater", "cast_rewrite resolve_theater_llm failed: {e}");
            return Ok(cast_rewrite_fallback_response(
                req,
                state.ollama_model.as_str(),
                "fallback",
                "rewrite_llm_resolve_failed",
            ));
        }
    };
    let source = resolve_llm_source_label(state);
    let timeout = Duration::from_secs(cast_rewrite_llm_timeout_secs());

    let persona_a = resolve_cast_persona(state, req.cast_a.role_id.as_str()).await;
    let persona_b = resolve_cast_persona(state, req.cast_b.role_id.as_str()).await;

    let prompt = crate::domain::theater_director::build_theater_prompt(
        state,
        &crate::domain::theater_director::cast_rewrite_prompt_input(
            req,
            min_beats,
            max_beats,
            false,
            persona_a.as_str(),
            persona_b.as_str(),
        ),
    );
    let raw = match tokio::time::timeout(timeout, llm.generate_tag(model.as_str(), prompt.as_str())).await
    {
        Ok(Ok(t)) => t,
        Ok(Err(e)) => {
            tracing::warn!(target: "oclive_theater", "cast_rewrite LLM failed: {e}");
            return Ok(cast_rewrite_fallback_response(req, &model, "fallback", "rewrite_llm_error"));
        }
        Err(_) => {
            tracing::warn!(
                target: "oclive_theater",
                "cast_rewrite LLM timed out ({}s)",
                cast_rewrite_llm_timeout_secs()
            );
            return Ok(cast_rewrite_fallback_response(req, &model, "fallback", "rewrite_llm_timeout"));
        }
    };

    if let Some(beats) = try_parse_cast_rewrite_beats_only(req, &raw, min_beats, max_beats) {
        return Ok(cast_rewrite_beats_only_response(req, beats, &model, source));
    }

    let target_beats = cast_rewrite_target_beats(min_beats, max_beats);
    let strict_prompt = crate::domain::theater_director::build_theater_prompt(
        state,
        &crate::domain::theater_director::cast_rewrite_minimal_prompt_input(
            req,
            target_beats,
            persona_a.as_str(),
            persona_b.as_str(),
        ),
    );
    let retry_raw = match tokio::time::timeout(
        timeout,
        llm.generate_tag(model.as_str(), strict_prompt.as_str()),
    )
    .await
    {
        Ok(Ok(t)) => t,
        Ok(Err(e)) => {
            tracing::warn!(target: "oclive_theater", "cast_rewrite retry LLM failed: {e}");
            return Ok(cast_rewrite_fallback_response(req, &model, "fallback", "rewrite_llm_error"));
        }
        Err(_) => {
            tracing::warn!(
                target: "oclive_theater",
                "cast_rewrite retry LLM timed out ({}s)",
                cast_rewrite_llm_timeout_secs()
            );
            return Ok(cast_rewrite_fallback_response(req, &model, "fallback", "rewrite_llm_timeout"));
        }
    };

    if let Some(beats) = try_parse_cast_rewrite_beats_only(req, &retry_raw, min_beats, max_beats) {
        return Ok(cast_rewrite_beats_only_response(req, beats, &model, source));
    }

    tracing::warn!(
        target: "oclive_theater",
        preview = %raw_preview_for_log(&retry_raw),
        "cast_rewrite parse failed; using fallback"
    );
    Ok(cast_rewrite_fallback_response(req, &model, "fallback", "rewrite_parse_failed"))
}

fn raw_preview_for_log(raw: &str) -> String {
    let collapsed: String = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.len() <= 240 {
        collapsed
    } else {
        format!("{}…", &collapsed[..240])
    }
}

fn cast_rewrite_fallback_response(
    req: &TheaterSceneRequest,
    model: &str,
    source: &str,
    failure_reason: &str,
) -> TheaterSceneResponse {
    scene_response_with_meta(
        req.fallback_beats.clone(),
        source,
        model.to_string(),
        req.fork_templates.clone(),
        Some(failure_reason),
        None,
    )
}

/// Full beats+forks parse (kept for tests / future opt-in path).
#[allow(dead_code)]
fn try_parse_cast_rewrite(
    req: &TheaterSceneRequest,
    poke_chips: &[TheaterPokeChipDef],
    raw: &str,
    min_beats: u32,
    max_beats: u32,
) -> Option<(Vec<TheaterScriptLine>, Vec<TheaterForkTemplate>)> {
    let parsed = parse_cast_rewrite_json(raw, min_beats, max_beats)?;
    let beats = normalize_rewrite_beats(req, &parsed.beats)?;
    let forks = normalize_rewrite_forks(req, poke_chips, &beats, &parsed.forks)?;
    Some((beats, forks))
}

/// Accept rewritten beats even when fork JSON is incomplete; keep name-bound fork templates.
fn try_parse_cast_rewrite_beats_only(
    req: &TheaterSceneRequest,
    raw: &str,
    min_beats: u32,
    max_beats: u32,
) -> Option<Vec<TheaterScriptLine>> {
    let ctx = CastRewriteParseCtx::from_req(req);
    let beats = parse_cast_rewrite_beats_loose_with_ctx(raw, min_beats, max_beats, &ctx)
        .or_else(|| salvage_rewrite_objects(raw, min_beats, max_beats, &ctx))?;
    normalize_rewrite_beats(req, &beats)
}

/// Parse beats from LLM output when forks are not required or fork JSON is invalid.
#[allow(dead_code)]
fn parse_cast_rewrite_beats_loose(
    raw: &str,
    min_beats: u32,
    max_beats: u32,
) -> Option<Vec<TheaterScriptLine>> {
    parse_cast_rewrite_beats_loose_with_ctx(raw, min_beats, max_beats, &CastRewriteParseCtx {
        name_a: String::new(),
        name_b: String::new(),
        role_id_a: String::new(),
        role_id_b: String::new(),
    })
}

fn parse_cast_rewrite_beats_loose_with_ctx(
    raw: &str,
    min_beats: u32,
    max_beats: u32,
    ctx: &CastRewriteParseCtx,
) -> Option<Vec<TheaterScriptLine>> {
    let trimmed = strip_code_fence(raw.trim());
    parse_cast_rewrite_beats_from_trimmed(trimmed, min_beats, max_beats, ctx).or_else(|| {
        let repaired = repair_json_loose(trimmed);
        if repaired == trimmed {
            None
        } else {
            parse_cast_rewrite_beats_from_trimmed(&repaired, min_beats, max_beats, ctx)
        }
    })
}

fn parse_cast_rewrite_beats_from_trimmed(
    trimmed: &str,
    min_beats: u32,
    max_beats: u32,
    ctx: &CastRewriteParseCtx,
) -> Option<Vec<TheaterScriptLine>> {
    if let Some(obj_slice) = extract_json_object(trimmed) {
        if let Ok(value) = serde_json::from_str::<Value>(obj_slice) {
            if let Some(beats_arr) = value.get("beats").and_then(|v| v.as_array()) {
                if let Some(beats) =
                    parse_rewrite_beats_array(beats_arr, min_beats, max_beats, Some(ctx))
                {
                    return Some(beats);
                }
            }
        }
    }

    let array_slice = extract_json_array(trimmed).unwrap_or(trimmed);
    if let Ok(value) = serde_json::from_str::<Value>(array_slice) {
        if let Some(arr) = value.as_array() {
            return parse_rewrite_beats_array(arr, min_beats, max_beats, Some(ctx));
        }
    }

    None
}

fn parse_rewrite_beats_array(
    arr: &[Value],
    min_beats: u32,
    max_beats: u32,
    ctx: Option<&CastRewriteParseCtx>,
) -> Option<Vec<TheaterScriptLine>> {
    if arr.len() < min_beats as usize || arr.len() > max_beats as usize {
        return None;
    }
    let mut beats = Vec::with_capacity(arr.len());
    let mut beat_ids = HashSet::new();
    for (i, item) in arr.iter().enumerate() {
        let mut line = parse_rewrite_line_object(item, ctx)?;
        if line.id.is_empty() || line.id == "beat" || beat_ids.contains(&line.id) {
            line.id = format!("b{}", i + 1);
        }
        beat_ids.insert(line.id.clone());
        beats.push(line);
    }
    Some(beats)
}

fn parse_rewrite_line_object(v: &Value, ctx: Option<&CastRewriteParseCtx>) -> Option<TheaterScriptLine> {
    let cast_raw = v
        .get("cast")
        .or_else(|| v.get("speaker"))
        .or_else(|| v.get("role"))
        .and_then(|x| x.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())?;
    let cast = resolve_rewrite_cast(cast_raw, ctx)?;
    let text = extract_rewrite_text(v)?;
    let id = v
        .get("id")
        .and_then(|x| x.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("beat")
        .to_string();
    let name = v
        .get("name")
        .and_then(|x| x.as_str())
        .map(str::trim)
        .unwrap_or("")
        .to_string();
    let stage_hint = v
        .get("stage_hint")
        .or_else(|| v.get("stageHint"))
        .and_then(|x| x.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty() && s.len() <= MAX_STAGE_HINT_LEN)
        .map(str::to_string);
    let emotion = v
        .get("emotion")
        .and_then(|x| x.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    Some(TheaterScriptLine {
        id,
        cast,
        name,
        text,
        stage_hint,
        emotion,
    })
}

fn extract_rewrite_text(v: &Value) -> Option<String> {
    for key in [
        "text", "dialogue", "line", "content", "utterance", "台词", "对白",
    ] {
        if let Some(text) = v.get(key).and_then(|x| x.as_str()).map(str::trim) {
            if text.is_empty() {
                continue;
            }
            if text.len() > MAX_BEAT_TEXT_LEN {
                return None;
            }
            return Some(text.to_string());
        }
    }
    None
}

fn resolve_rewrite_cast(raw: &str, ctx: Option<&CastRewriteParseCtx>) -> Option<String> {
    let lower = raw.to_ascii_lowercase();
    if matches!(
        lower.as_str(),
        "a" | "cast_a" | "casta" | "left" | "1" | "first" | "甲"
    ) {
        return Some("a".to_string());
    }
    if matches!(
        lower.as_str(),
        "b" | "cast_b" | "castb" | "right" | "2" | "second" | "乙"
    ) {
        return Some("b".to_string());
    }
    let ctx = ctx?;
    if raw.eq_ignore_ascii_case(ctx.name_a.as_str()) || raw == ctx.role_id_a.as_str() {
        return Some("a".to_string());
    }
    if raw.eq_ignore_ascii_case(ctx.name_b.as_str()) || raw == ctx.role_id_b.as_str() {
        return Some("b".to_string());
    }
    None
}

fn repair_json_loose(raw: &str) -> String {
    let mut s = raw.trim().to_string();
    for ch in ['\u{201c}', '\u{201d}', '\u{2018}', '\u{2019}'] {
        s = s.replace(ch, "\"");
    }
    s = remove_trailing_commas_before_close(&s);
    if s.contains('[') && !s.contains(']') {
        s.push(']');
    }
    s
}

fn remove_trailing_commas_before_close(raw: &str) -> String {
    let bytes = raw.as_bytes();
    let mut out = String::with_capacity(raw.len());
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b',' {
            let mut j = i + 1;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if j < bytes.len() && (bytes[j] == b']' || bytes[j] == b'}') {
                i += 1;
                continue;
            }
        }
        out.push(char::from(bytes[i]));
        i += 1;
    }
    out
}

fn salvage_rewrite_objects(
    raw: &str,
    min_beats: u32,
    max_beats: u32,
    ctx: &CastRewriteParseCtx,
) -> Option<Vec<TheaterScriptLine>> {
    let trimmed = strip_code_fence(raw.trim());
    let search = extract_json_array(trimmed).unwrap_or(trimmed);
    let mut beats = Vec::new();
    let mut beat_ids = HashSet::new();
    let mut i = 0;
    while i < search.len() {
        if search.as_bytes().get(i) == Some(&b'{') {
            if let Some((obj_slice, next)) = extract_balanced_json_object(search, i) {
                if let Ok(value) = serde_json::from_str::<Value>(obj_slice) {
                    if let Some(mut line) = parse_rewrite_line_object(&value, Some(ctx)) {
                        if line.id.is_empty() || line.id == "beat" || beat_ids.contains(&line.id) {
                            line.id = format!("b{}", beats.len() + 1);
                        }
                        beat_ids.insert(line.id.clone());
                        beats.push(line);
                    }
                }
                i = next;
                continue;
            }
        }
        i += 1;
    }
    if beats.len() >= min_beats as usize && beats.len() <= max_beats as usize {
        Some(beats)
    } else {
        None
    }
}

fn extract_balanced_json_object(s: &str, start: usize) -> Option<(&str, usize)> {
    if s.as_bytes().get(start) != Some(&b'{') {
        return None;
    }
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escape = false;
    for (i, b) in s.as_bytes().iter().enumerate().skip(start) {
        if in_string {
            if escape {
                escape = false;
            } else if *b == b'\\' {
                escape = true;
            } else if *b == b'"' {
                in_string = false;
            }
            continue;
        }
        match b {
            b'"' => in_string = true,
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some((&s[start..=i], i + 1));
                }
            }
            _ => {}
        }
    }
    None
}

fn cast_rewrite_beats_only_response(
    req: &TheaterSceneRequest,
    beats: Vec<TheaterScriptLine>,
    model: &str,
    source: &str,
) -> TheaterSceneResponse {
    scene_response_with_meta(
        beats,
        source,
        model.to_string(),
        req.fork_templates.clone(),
        None,
        Some("rewrite_forks_template"),
    )
}

fn pair_relation_block(req: &TheaterSceneRequest) -> String {
    let hint = req
        .pair_relation_hint
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let Some(hint) = hint else {
        return String::new();
    };
    let id = req
        .pair_relation_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("custom");
    format!("\n双角色关系（{id}）：{hint}\n")
}

fn default_scene_brief() -> &'static str {
    "早餐 · 上学前：厨房餐桌、温粥、收拾书包、出门前的日常照应与拌嘴。"
}

fn default_scene_setting_hint() -> &'static str {
    "地点限于家中厨房/餐桌/玄关；时间早晨上学前；禁止脱离居家早饭场景或引入第三人。"
}

fn scene_context_block(req: &TheaterSceneRequest) -> String {
    let brief = req
        .scene_brief
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(default_scene_brief());
    let setting = req
        .scene_setting_hint
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(default_scene_setting_hint());
    format!("场景：{brief}\n场景约束：{setting}\n")
}

fn cast_rewrite_requires_forks(req: &TheaterSceneRequest) -> bool {
    req.poke_chips
        .as_ref()
        .is_some_and(|chips| !chips.is_empty())
}

pub(crate) fn build_cast_rewrite_prompt(
    req: &TheaterSceneRequest,
    min_beats: u32,
    max_beats: u32,
    strict: bool,
    persona_a: &str,
    persona_b: &str,
) -> String {
    let strict_tail = if strict {
        "\n【严格】只输出 JSON 数组，无 Markdown、无解释。每条仅 id、cast、text；cast 只能是 a 或 b。"
    } else {
        ""
    };
    let persona_block = format!(
        "- A({name_a}): {pa}\n- B({name_b}): {pb}",
        name_a = req.cast_a.name.trim(),
        name_b = req.cast_b.name.trim(),
        pa = if persona_a.trim().is_empty() {
            "（无额外人设，按角色名推断高中生语气）"
        } else {
            persona_a.trim()
        },
        pb = if persona_b.trim().is_empty() {
            "（无额外人设，按角色名推断高中生语气）"
        } else {
            persona_b.trim()
        },
    );
    let relation_block = pair_relation_block(req);
    let scene_block = scene_context_block(req);
    let theater_scene = req
        .theater_scene
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("breakfast");
    let target_beats = cast_rewrite_target_beats(min_beats, max_beats);

    let poke_space_line = if cast_rewrite_requires_forks(req) {
        "\n5. 主剧本须为戳点 chip 可能触发的事件留出合理插入空间（中段附近可接小插曲），不要预写 forks 正文。"
    } else {
        ""
    };
    let guardrails = drama_guardrails::drama_guardrails_compact(req.theater_scene.as_deref());

    format!(
        r#"卡司重写：为以下两位角色**从零**撰写「{theater_scene}」双人短剧。不要沿用任何现成台词或剧情模板，须完全贴合人设关系与说话方式。

cast a={name_a}，cast b={name_b}，角色包场景={scene_id}。仅 a/b 两人发言，禁止第三人。

{scene_block}{guardrails}人设摘要：
{persona_block}{relation_block}
撰写要求：
1. 恰好 {target_beats} 条对白（id 依次为 b1,b2,b3…）；cast 只能是 a 或 b；text 非空≤{max_text}字；写**全新**对白与小事件，须落在上述场景约束内。
2. 开场 2 拍须建立场景物件感与两人性格对照；交替发言、有戏感、口语自然中文。
3. 中段留 poke 插入空间，勿把戳点事件写死进主剧本。{poke_space_line}
4. 戳点分支由系统另行挂载，不要输出 forks 字段。

输出格式（仅 JSON 数组，不要其它文字）：
- 只输出一个 JSON 数组；不要 Markdown、不要代码块围栏、不要前后说明。
- 每条仅含 id、cast、text 三个字段；不要 name、stage_hint、emotion 等字段。
- 示例：
[{{"id":"b1","cast":"b","text":"……"}},{{"id":"b2","cast":"a","text":"……"}}]
{strict_tail}"#,
        theater_scene = theater_scene,
        name_a = req.cast_a.name.trim(),
        name_b = req.cast_b.name.trim(),
        scene_id = req.scene_id.trim(),
        scene_block = scene_block,
        guardrails = guardrails,
        persona_block = persona_block,
        relation_block = relation_block,
        target_beats = target_beats,
        max_text = MAX_BEAT_TEXT_LEN,
        strict_tail = strict_tail,
        poke_space_line = poke_space_line,
    )
}

pub(crate) fn build_cast_rewrite_minimal_prompt(
    req: &TheaterSceneRequest,
    target_beats: u32,
    persona_a: &str,
    persona_b: &str,
) -> String {
    let brief = req
        .scene_brief
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(default_scene_brief());
    let pa = if persona_a.trim().is_empty() {
        "按角色名推断语气"
    } else {
        persona_a.trim()
    };
    let pb = if persona_b.trim().is_empty() {
        "按角色名推断语气"
    } else {
        persona_b.trim()
    };
    let guardrails = drama_guardrails::drama_guardrails_compact(req.theater_scene.as_deref());
    format!(
        r#"只输出 JSON 数组，恰好 {target} 条对白。从 [ 开始到 ] 结束，不要 Markdown、不要解释。
cast 只能是 a 或 b；每条仅 id、cast、text 三个字段。
A({name_a})={pa}
B({name_b})={pb}
场景：{brief}{guardrails}
示例：[{{"id":"b1","cast":"b","text":"……"}},{{"id":"b2","cast":"a","text":"……"}},{{"id":"b3","cast":"a","text":"……"}}]"#,
        target = target_beats,
        name_a = req.cast_a.name.trim(),
        name_b = req.cast_b.name.trim(),
        pa = pa,
        pb = pb,
        brief = brief,
        guardrails = guardrails,
    )
}

pub struct CastRewriteLlmFork {
    pub chip_id: String,
    pub insert_after_beat_id: String,
    pub patch_lines: Vec<TheaterScriptLine>,
}

pub struct CastRewriteLlmOutput {
    pub beats: Vec<TheaterScriptLine>,
    pub forks: Vec<CastRewriteLlmFork>,
}

/// Parse cast-rewrite LLM JSON object.
#[must_use]
pub fn parse_cast_rewrite_json(
    raw: &str,
    min_beats: u32,
    max_beats: u32,
) -> Option<CastRewriteLlmOutput> {
    let trimmed = strip_code_fence(raw.trim());
    let json_slice = extract_json_object(trimmed).unwrap_or(trimmed);
    let value: Value = serde_json::from_str(json_slice).ok()?;
    let obj = value.as_object()?;

    let beats_arr = obj.get("beats")?.as_array()?;
    if beats_arr.len() < min_beats as usize || beats_arr.len() > max_beats as usize {
        return None;
    }
    let mut beats = Vec::with_capacity(beats_arr.len());
    let mut beat_ids = HashSet::new();
    for item in beats_arr {
        let line = parse_line_object(item)?;
        if !beat_ids.insert(line.id.clone()) {
            return None;
        }
        beats.push(line);
    }

    let forks_arr = obj
        .get("forks")
        .and_then(|v| v.as_array())
        .map_or_else(Vec::new, |arr| arr.to_vec());
    let mut forks = Vec::with_capacity(forks_arr.len());
    for item in &forks_arr {
        let chip_id = item
            .get("chip_id")
            .or_else(|| item.get("chipId"))
            .and_then(|x| x.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())?
            .to_string();
        let insert_after = item
            .get("insert_after_beat_id")
            .or_else(|| item.get("insertAfterBeatId"))
            .and_then(|x| x.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())?
            .to_string();
        let patch_arr = item.get("patch_lines").or_else(|| item.get("patchLines"))?.as_array()?;
        if patch_arr.is_empty() || patch_arr.len() > 8 {
            return None;
        }
        let mut patch_lines = Vec::with_capacity(patch_arr.len());
        for line in patch_arr {
            patch_lines.push(parse_line_object(line)?);
        }
        forks.push(CastRewriteLlmFork {
            chip_id,
            insert_after_beat_id: insert_after,
            patch_lines,
        });
    }

    Some(CastRewriteLlmOutput { beats, forks })
}

fn normalize_rewrite_beats(
    req: &TheaterSceneRequest,
    llm_beats: &[TheaterScriptLine],
) -> Option<Vec<TheaterScriptLine>> {
    let mut out = Vec::with_capacity(llm_beats.len());
    for line in llm_beats {
        let cast = line.cast.trim().to_ascii_lowercase();
        if cast != "a" && cast != "b" {
            return None;
        }
        let name = if cast == "a" {
            req.cast_a.name.trim().to_string()
        } else {
            req.cast_b.name.trim().to_string()
        };
        out.push(TheaterScriptLine {
            id: line.id.clone(),
            cast,
            name,
            text: line.text.clone(),
            stage_hint: line.stage_hint.clone(),
            emotion: line.emotion.clone(),
        });
    }
    Some(out)
}

#[allow(dead_code)]
fn normalize_rewrite_forks(
    req: &TheaterSceneRequest,
    poke_chips: &[TheaterPokeChipDef],
    beats: &[TheaterScriptLine],
    llm_forks: &[CastRewriteLlmFork],
) -> Option<Vec<TheaterForkTemplate>> {
    if poke_chips.is_empty() {
        return Some(Vec::new());
    }
    let beat_ids: HashSet<&str> = beats.iter().map(|b| b.id.as_str()).collect();
    let llm_by_chip: HashMap<&str, &CastRewriteLlmFork> =
        llm_forks.iter().map(|f| (f.chip_id.as_str(), f)).collect();

    let mut out = Vec::with_capacity(poke_chips.len());
    for chip in poke_chips {
        let chip_id = chip.chip_id.trim();
        if chip_id.is_empty() {
            return None;
        }
        let llm_fork = llm_by_chip.get(chip_id)?;
        if !beat_ids.contains(llm_fork.insert_after_beat_id.as_str()) {
            return None;
        }
        let patch_lines: Vec<TheaterScriptLine> = llm_fork
            .patch_lines
            .iter()
            .map(|line| {
                let cast = line.cast.trim().to_ascii_lowercase();
                let name = if cast == "a" {
                    req.cast_a.name.trim().to_string()
                } else if cast == "b" {
                    req.cast_b.name.trim().to_string()
                } else {
                    line.name.clone()
                };
                TheaterScriptLine {
                    id: line.id.clone(),
                    cast: line.cast.clone(),
                    name,
                    text: line.text.clone(),
                    stage_hint: line.stage_hint.clone(),
                    emotion: line.emotion.clone(),
                }
            })
            .collect();
        out.push(TheaterForkTemplate {
            chip_id: chip_id.to_string(),
            insert_after_beat_id: llm_fork.insert_after_beat_id.clone(),
            patch_lines,
        });
    }
    Some(out)
}

fn cast_adapt_fallback_response(
    req: &TheaterSceneRequest,
    model: &str,
    source: &str,
) -> TheaterSceneResponse {
    scene_response_with_meta(
        req.fallback_beats.clone(),
        source,
        model.to_string(),
        req.fork_templates.clone(),
        Some("adapt_parse_failed"),
        None,
    )
}

fn try_merge_cast_adapt(
    req: &TheaterSceneRequest,
    templates: &[TheaterForkTemplate],
    raw: &str,
    max_beats: u32,
) -> Option<(Vec<TheaterScriptLine>, Vec<TheaterForkTemplate>)> {
    let parsed = parse_cast_adapt_json(raw, max_beats)?;
    let beats = merge_adapted_beats(&req.base_beats, &parsed.beats);
    let forks = merge_adapted_forks(templates, &parsed.forks);
    Some((beats, forks))
}

pub struct CastAdaptLlmFork {
    pub chip_id: String,
    pub patch_lines: Vec<TheaterScriptLine>,
}

pub struct CastAdaptLlmOutput {
    pub beats: Vec<TheaterScriptLine>,
    pub forks: Vec<CastAdaptLlmFork>,
}

/// Build cast-adapt instruction prompt (optionally stricter on retry).
#[must_use]
pub(crate) fn build_cast_adapt_prompt(
    req: &TheaterSceneRequest,
    templates: &[TheaterForkTemplate],
    max_beats: u32,
    strict: bool,
    persona_a: &str,
    persona_b: &str,
) -> String {
    let beats_json = serde_json::to_string(&req.base_beats).unwrap_or_else(|_| "[]".to_string());
    let forks_json = serde_json::to_string(templates).unwrap_or_else(|_| "[]".to_string());

    let strict_tail = if strict {
        "\n【严格】只输出 JSON 对象，无 Markdown、无解释。每个 beat/fork patch 的 id 与 cast 必须与骨架完全一致。"
    } else {
        ""
    };

    let pass_block = cast_adapt_pass_instructions(req.adapt_pass.as_deref());

    let persona_block = if persona_a.trim().is_empty() && persona_b.trim().is_empty() {
        String::new()
    } else {
        format!(
            "\n人设摘要（语气/性格约束）：\n- A({name_a}): {pa}\n- B({name_b}): {pb}\n",
            name_a = req.cast_a.name.trim(),
            name_b = req.cast_b.name.trim(),
            pa = if persona_a.trim().is_empty() {
                "（无）"
            } else {
                persona_a.trim()
            },
            pb = if persona_b.trim().is_empty() {
                "（无）"
            } else {
                persona_b.trim()
            },
        )
    };

    let scene_block = scene_context_block(req);
    let guardrails = drama_guardrails::drama_guardrails_compact(req.theater_scene.as_deref());

    format!(
        r#"卡司适配：双人剧场。cast a={name_a}，cast b={name_b}，场景={scene_id}。仅 a/b 发言。
{pass_block}{guardrails}{scene_block}{persona_block}
开场 beats 骨架（id/cast 只读，可改 name/text/stage_hint/emotion）：
{beats_json}

戳点 fork 罐头（每项 chip_id 只读；patch_lines 的 id/cast 只读，可改 name/text/stage_hint/emotion；勿输出 insert_after_beat_id）：
{forks_json}

输出契约：JSON 对象 {{"beats":[...],"forks":[{{"chip_id","patch_lines":[...]}}]}}（forks 可省略，有则改写戳点罐头）。
规则：beats 每项 id/cast 与骨架一致；forks 每项 chip_id 与骨架一致、patch_lines 的 id/cast 一致；总 beats≤{max_beats}；text 非空≤{max_text}字；台词须明显贴合各角色人设；禁止仅替换姓名；不得新增第三人或自造 id。

示例：{{"beats":[{{"id":"b1","cast":"b","name":"{name_b}","text":"……","emotion":"happy"}}],"forks":[{{"chip_id":"tea","patch_lines":[{{"id":"tea-1","cast":"b","name":"{name_b}","text":"……"}}]}}]}}{strict_tail}
"#,
        name_a = req.cast_a.name.trim(),
        name_b = req.cast_b.name.trim(),
        scene_id = req.scene_id.trim(),
        pass_block = pass_block,
        guardrails = guardrails,
        scene_block = scene_block,
        persona_block = persona_block,
        beats_json = beats_json,
        forks_json = forks_json,
        max_beats = max_beats,
        max_text = MAX_BEAT_TEXT_LEN,
        strict_tail = strict_tail,
    )
}

fn cast_adapt_pass_instructions(pass: Option<&str>) -> String {
    let focus = match pass.map(str::trim).filter(|s| !s.is_empty()) {
        Some("voice") => {
            "【本轮·语气人设】第一轮：在保持事件顺序与 beat id 不变的前提下，把每位角色的台词改成其人设口吻；同步调整 emotion/stage_hint 以贴合性格（毒舌/温柔/别扭等）。禁止只改姓名。"
        }
        Some("depth") => {
            "【本轮·角色化大纲】第二轮：在当前场景时间框架内，进一步改写台词内容与 stage_hint，使互动、拌嘴方式、关心/抵触的表达方式更符合两位角色的关系与性格；可调整具体物件与情绪转折，但 beat id/cast 不可变，仍须落在同一 scene_brief 场景。"
        }
        Some("polish") => {
            "【本轮·戳点收束】第三轮：重点改写 forks 戳点罐头台词，每条须是可分享的一击（有反差/动作/情绪），勿平述；beats 做最终通顺与人设一致性润色，确保全剧台词风格统一、角色区分度明显。"
        }
        _ => {
            "【综合适配】语气、角色化互动与戳点一并改写；beat id/cast 不可变。"
        }
    };
    format!("\n{focus}\n")
}

/// Parse cast-adapt LLM JSON object; returns `None` when invalid.
#[must_use]
pub fn parse_cast_adapt_json(raw: &str, max_beats: u32) -> Option<CastAdaptLlmOutput> {
    let trimmed = strip_code_fence(raw.trim());
    let json_slice = extract_json_object(trimmed).unwrap_or(trimmed);
    let value: Value = serde_json::from_str(json_slice).ok()?;
    let obj = value.as_object()?;

    let beats_arr = obj.get("beats")?.as_array()?;
    if beats_arr.is_empty() || beats_arr.len() > max_beats as usize {
        return None;
    }
    let mut beats = Vec::with_capacity(beats_arr.len());
    for item in beats_arr {
        beats.push(parse_line_object(item)?);
    }

    let forks_arr = obj
        .get("forks")
        .and_then(|v| v.as_array())
        .map_or_else(Vec::new, |arr| arr.to_vec());
    let mut forks = Vec::with_capacity(forks_arr.len());
    for item in &forks_arr {
        let chip_id = item
            .get("chip_id")
            .or_else(|| item.get("chipId"))
            .and_then(|x| x.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())?
            .to_string();
        let patch_arr = item.get("patch_lines").or_else(|| item.get("patchLines"))?.as_array()?;
        let mut patch_lines = Vec::with_capacity(patch_arr.len());
        for line in patch_arr {
            patch_lines.push(parse_line_object(line)?);
        }
        forks.push(CastAdaptLlmFork {
            chip_id,
            patch_lines,
        });
    }

    Some(CastAdaptLlmOutput { beats, forks })
}

/// Merge LLM-adapted beats onto skeleton by id; invalid lines fall back to skeleton.
#[must_use]
pub fn merge_adapted_beats(
    skeleton: &[TheaterScriptLine],
    llm_beats: &[TheaterScriptLine],
) -> Vec<TheaterScriptLine> {
    let llm_by_id: HashMap<&str, &TheaterScriptLine> =
        llm_beats.iter().map(|b| (b.id.as_str(), b)).collect();
    skeleton
        .iter()
        .map(|base| {
            llm_by_id
                .get(base.id.as_str())
                .map(|adapted| merge_adapted_line(base, adapted))
                .unwrap_or_else(|| base.clone())
        })
        .collect()
}

/// Merge LLM-adapted fork patches onto templates by chip_id + patch line id.
#[must_use]
pub fn merge_adapted_forks(
    templates: &[TheaterForkTemplate],
    llm_forks: &[CastAdaptLlmFork],
) -> Vec<TheaterForkTemplate> {
    let llm_by_chip: HashMap<&str, &CastAdaptLlmFork> =
        llm_forks.iter().map(|f| (f.chip_id.as_str(), f)).collect();
    templates
        .iter()
        .map(|tmpl| {
            let patch_lines = if let Some(llm_fork) = llm_by_chip.get(tmpl.chip_id.as_str()) {
                let llm_by_id: HashMap<&str, &TheaterScriptLine> = llm_fork
                    .patch_lines
                    .iter()
                    .map(|l| (l.id.as_str(), l))
                    .collect();
                tmpl.patch_lines
                    .iter()
                    .map(|base| {
                        llm_by_id
                            .get(base.id.as_str())
                            .map(|adapted| merge_adapted_line(base, adapted))
                            .unwrap_or_else(|| base.clone())
                    })
                    .collect()
            } else {
                tmpl.patch_lines.clone()
            };
            TheaterForkTemplate {
                chip_id: tmpl.chip_id.clone(),
                insert_after_beat_id: tmpl.insert_after_beat_id.clone(),
                patch_lines,
            }
        })
        .collect()
}

fn merge_adapted_line(
    base: &TheaterScriptLine,
    adapted: &TheaterScriptLine,
) -> TheaterScriptLine {
    if adapted.id != base.id || adapted.cast != base.cast {
        return base.clone();
    }
    let text = adapted.text.trim();
    if text.is_empty() || text.len() > MAX_BEAT_TEXT_LEN {
        return base.clone();
    }
    let name = adapted.name.trim();
    TheaterScriptLine {
        id: base.id.clone(),
        cast: base.cast.clone(),
        name: if name.is_empty() {
            base.name.clone()
        } else {
            name.to_string()
        },
        text: text.to_string(),
        stage_hint: adapted
            .stage_hint
            .clone()
            .filter(|s| s.len() <= MAX_STAGE_HINT_LEN)
            .or_else(|| base.stage_hint.clone()),
        emotion: adapted.emotion.clone().or_else(|| base.emotion.clone()),
    }
}

fn extract_json_object(raw: &str) -> Option<&str> {
    let start = raw.find('{')?;
    let end = raw.rfind('}')?;
    if end <= start {
        return None;
    }
    Some(&raw[start..=end])
}

/// Resolve the effective LLM client + model for theater scene generation (local Ollama or cloud BYOK).
pub(crate) async fn resolve_theater_llm(
    state: &AppState,
    req: &TheaterSceneRequest,
) -> Result<(Arc<dyn LlmClient>, String)> {
    let session_ns = format!("theater:{}", req.scene_id.trim());
    let role = if let Ok(role) = state.load_role_cached_async(req.cast_a.role_id.trim()).await {
        role
    } else if let Ok(role) = state.load_role_cached_async(req.cast_b.role_id.trim()).await {
        role
    } else {
        return Err(AppError::InvalidParameter(format!(
            "theater scene: neither cast role loaded (a={}, b={})",
            req.cast_a.role_id.trim(),
            req.cast_b.role_id.trim()
        )));
    };
    let backends = state.effective_plugin_backends_for_session(role.as_ref(), session_ns.as_str());
    let llm = state.plugins.llm_for_plugin_backends(backends.as_ref());
    let model =
        resolve_effective_ollama_model(state, role.as_ref(), session_ns.as_str()).await?;
    Ok((llm, model))
}

pub(crate) fn resolve_llm_source_label(state: &AppState) -> &'static str {
    let provider = state.user_llm_provider.read().trim().to_ascii_lowercase();
    if provider == "cloud" {
        "cloud"
    } else {
        "local"
    }
}

const PERSONA_DESC_MAX: usize = 200;
const PERSONA_CORE_MAX: usize = 280;
const PERSONA_PROMPT_MAX: usize = 300;

/// Load a short persona summary from role pack (description + core + optional prompt snippet).
pub(crate) async fn resolve_cast_persona(state: &AppState, role_id: &str) -> String {
    let role_id = role_id.trim();
    if role_id.is_empty() {
        return String::new();
    }
    let Ok(role) = state.load_role_cached_async(role_id).await else {
        return String::new();
    };
    let mut parts: Vec<String> = Vec::new();
    let desc = role.description.trim();
    if !desc.is_empty() {
        parts.push(truncate_chars(desc, PERSONA_DESC_MAX));
    }
    let core = role.core_personality.trim();
    if !core.is_empty() {
        parts.push(truncate_chars(core, PERSONA_CORE_MAX));
    }
    if let Some(extra) = read_role_prompt_snippet(state, role_id) {
        parts.push(extra);
    }
    parts.join(" ")
}

fn read_role_prompt_snippet(state: &AppState, role_id: &str) -> Option<String> {
    let role_dir = state.storage.roles_dir().join(role_id);
    for rel in [
        "prompts/identity.md",
        "prompts/character.md",
        "identity.md",
        "character.md",
    ] {
        let path = role_dir.join(rel);
        if !path.is_file() {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let snippet = first_prompt_paragraph(&text);
        if !snippet.is_empty() {
            return Some(truncate_chars(&snippet, PERSONA_PROMPT_MAX));
        }
    }
    None
}

fn first_prompt_paragraph(text: &str) -> String {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .take(3)
        .collect::<Vec<_>>()
        .join(" ")
}

fn truncate_chars(s: &str, max: usize) -> String {
    let t = s.trim();
    if t.chars().count() <= max {
        t.to_string()
    } else {
        format!("{}…", t.chars().take(max).collect::<String>())
    }
}

/// Beats in the base script that follow the insert anchor (ripple skeleton reference).
#[must_use]
pub(crate) fn beats_after_insert(
    base_beats: &[TheaterScriptLine],
    insert_after_beat_id: &str,
) -> Vec<TheaterScriptLine> {
    let anchor = insert_after_beat_id.trim();
    let Some(idx) = base_beats.iter().position(|b| b.id == anchor) else {
        return Vec::new();
    };
    base_beats[idx + 1..].to_vec()
}

/// Split fallback working script into immutable prefix and ripple skeleton.
#[must_use]
fn resolve_ripple_context(req: &TheaterSceneRequest) -> RippleContext {
    if req.applied_tweaks.is_empty() {
        return RippleContext {
            prefix_beats: Vec::new(),
            ripple_skeleton: req.base_beats.clone(),
            full_rewrite: true,
        };
    }

    let anchor = req
        .applied_tweaks
        .last()
        .map(|t| t.insert_after_beat_id.trim())
        .unwrap_or("");
    let ripple_skeleton = beats_after_insert(&req.base_beats, anchor);

    if ripple_skeleton.is_empty() {
        return RippleContext {
            prefix_beats: req.fallback_beats.clone(),
            ripple_skeleton: Vec::new(),
            full_rewrite: false,
        };
    }

    let first_ripple_id = ripple_skeleton[0].id.as_str();
    let split_idx = req
        .fallback_beats
        .iter()
        .position(|b| b.id == first_ripple_id);
    let prefix_beats = match split_idx {
        Some(i) => req.fallback_beats[..i].to_vec(),
        None => req.fallback_beats.clone(),
    };

    RippleContext {
        prefix_beats,
        ripple_skeleton,
        full_rewrite: false,
    }
}

/// Merge prefix beats with parsed LLM ripple output.
#[must_use]
pub(crate) fn merge_scene_beats(
    prefix: &[TheaterScriptLine],
    ripple: Vec<TheaterScriptLine>,
) -> Vec<TheaterScriptLine> {
    let mut out = prefix.to_vec();
    out.extend(ripple);
    out
}

pub(crate) fn ripple_ids_conflict(prefix: &[TheaterScriptLine], ripple: &[TheaterScriptLine]) -> bool {
    let prefix_ids: HashSet<&str> = prefix.iter().map(|b| b.id.as_str()).collect();
    ripple.iter().any(|b| prefix_ids.contains(b.id.as_str()))
}

fn try_merge_ripple(
    ctx: &RippleContext,
    raw: &str,
    max_beats: u32,
) -> Option<Vec<TheaterScriptLine>> {
    let ripple = parse_scene_json(raw, max_beats)?;
    if ripple_ids_conflict(&ctx.prefix_beats, &ripple) {
        return None;
    }
    if ctx.full_rewrite {
        Some(ripple)
    } else if ctx.ripple_skeleton.is_empty() {
        Some(ctx.prefix_beats.clone())
    } else {
        Some(merge_scene_beats(&ctx.prefix_beats, ripple))
    }
}

/// Build the scene-director instruction prompt (optionally stricter on retry).
#[must_use]
pub(crate) fn build_scene_prompt(
    req: &TheaterSceneRequest,
    ctx: &RippleContext,
    max_beats: u32,
    strict: bool,
    persona_a: &str,
    persona_b: &str,
) -> String {
    let prefix_json = serde_json::to_string(&ctx.prefix_beats).unwrap_or_else(|_| "[]".to_string());
    let ripple_json =
        serde_json::to_string(&ctx.ripple_skeleton).unwrap_or_else(|_| "[]".to_string());
    let tweaks_json =
        serde_json::to_string(&req.applied_tweaks).unwrap_or_else(|_| "[]".to_string());

    let scope_block = if ctx.full_rewrite {
        format!(
            "无微调：重写整场（≤{max_beats} 拍）。开场骨架：\n{ripple_json}"
        )
    } else {
        format!(
            "前缀（只读，禁止改写或重复输出）：\n{prefix_json}\n\n涟漪区骨架（须重写，体现 drama_seed）：\n{ripple_json}"
        )
    };

    let tweak_block = if req.applied_tweaks.is_empty() {
        "（无微调）".to_string()
    } else {
        format!(
            "微调意图：{tweaks_json}\n\
微调纪律：drama_seed 是剧情变数/事件，须融入涟漪区大纲；\
不得机械复制罐头 fork 的 cast 顺序或台词；\
由谁开口、谁主导反应由 A/B 人设摘要与前缀上下文决定，两人都要有戏；\
罐头 patchLines 仅表事件方向与接锚点，不是强制台词模板；\
须自然接回后续节拍走向。涟漪区须比前缀更有张力，禁止平淡续写。"
        )
    };

    let strict_tail = if strict {
        "\n【严格】只输出 JSON 数组，无 Markdown、无解释。"
    } else {
        ""
    };

    let persona_block = if persona_a.trim().is_empty() && persona_b.trim().is_empty() {
        String::new()
    } else {
        format!(
            "\n人设摘要（语气/性格约束，不得改变大纲事件）：\n- A({name_a}): {pa}\n- B({name_b}): {pb}\n",
            name_a = req.cast_a.name.trim(),
            name_b = req.cast_b.name.trim(),
            pa = if persona_a.trim().is_empty() {
                "（无）"
            } else {
                persona_a.trim()
            },
            pb = if persona_b.trim().is_empty() {
                "（无）"
            } else {
                persona_b.trim()
            },
        )
    };
    let relation_block = pair_relation_block(req);
    let scene_block = scene_context_block(req);
    let setting_tail = req
        .scene_setting_hint
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(default_scene_setting_hint());

    let guardrails = drama_guardrails::drama_guardrails_full(req.theater_scene.as_deref());

    format!(
        r#"场景导演：双人剧场。cast a={name_a}，cast b={name_b}，角色包场景={scene_id}。仅 a/b 发言。
{persona_block}{relation_block}{scene_block}{guardrails}
{scope_block}

{tweak_block}

输出契约：JSON 数组，每元素 {{"id","cast":"a"|"b","name","text","stage_hint?","emotion?"}}。
规则：只输出{output_scope}；总拍数≤{max_beats}；text 非空≤{max_text}字；name 与 cast 一致；台词须符合各人设；\
微调时 cast 分配须随人设与上下文决定，勿照搬罐头 fork 的说话顺序；{setting_tail}；不得新增第三人。

示例：[{{"id":"r1","cast":"b","name":"枫侵月","text":"……","stage_hint":"推碗","emotion":"happy"}},{{"id":"r2","cast":"a","name":"木木","text":"……","emotion":"shy"}}]{strict_tail}
"#,
        name_a = req.cast_a.name.trim(),
        name_b = req.cast_b.name.trim(),
        scene_id = req.scene_id.trim(),
        persona_block = persona_block,
        relation_block = relation_block,
        scene_block = scene_block,
        guardrails = guardrails,
        scope_block = scope_block,
        tweak_block = tweak_block,
        output_scope = if ctx.full_rewrite {
            "整场"
        } else {
            "涟漪区（不含前缀）"
        },
        max_beats = max_beats,
        max_text = MAX_BEAT_TEXT_LEN,
        setting_tail = setting_tail,
        strict_tail = strict_tail,
    )
}

/// Parse and validate LLM output into beats; returns `None` when invalid.
#[must_use]
pub fn parse_scene_json(raw: &str, max_beats: u32) -> Option<Vec<TheaterScriptLine>> {
    let trimmed = strip_code_fence(raw.trim());
    let json_slice = extract_json_array(trimmed).unwrap_or(trimmed);
    let value: Value = serde_json::from_str(json_slice).ok()?;
    let arr = value.as_array()?;
    if arr.is_empty() || arr.len() > max_beats as usize {
        return None;
    }

    let mut beats = Vec::with_capacity(arr.len());
    for item in arr {
        let line = parse_line_object(item)?;
        beats.push(line);
    }
    Some(beats)
}

fn parse_line_object(v: &Value) -> Option<TheaterScriptLine> {
    let cast = v.get("cast")?.as_str()?.trim().to_ascii_lowercase();
    if cast != "a" && cast != "b" {
        return None;
    }
    let text = v.get("text")?.as_str()?.trim();
    if text.is_empty() || text.len() > MAX_BEAT_TEXT_LEN {
        return None;
    }
    let id = v
        .get("id")
        .and_then(|x| x.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("beat");
    let name = v
        .get("name")
        .and_then(|x| x.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())?;
    let stage_hint = v
        .get("stage_hint")
        .or_else(|| v.get("stageHint"))
        .and_then(|x| x.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty() && s.len() <= MAX_STAGE_HINT_LEN)
        .map(str::to_string);
    let emotion = v
        .get("emotion")
        .and_then(|x| x.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    Some(TheaterScriptLine {
        id: id.to_string(),
        cast,
        name: name.to_string(),
        text: text.to_string(),
        stage_hint,
        emotion,
    })
}

fn strip_code_fence(raw: &str) -> &str {
    let s = raw.trim();
    if !s.starts_with("```") {
        return s;
    }
    let inner = s.trim_start_matches('`').trim_start();
    let after_lang = inner.find('\n').map(|i| &inner[i + 1..]).unwrap_or(inner);
    after_lang
        .trim_end_matches('`')
        .trim()
}

fn extract_json_array(raw: &str) -> Option<&str> {
    let start = raw.find('[')?;
    let end = raw.rfind(']')?;
    if end <= start {
        return None;
    }
    Some(&raw[start..=end])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::dto::{TheaterCastRef, TheaterForkTemplate, TheaterPokeChipDef, TheaterTweak};

    fn sample_req() -> TheaterSceneRequest {
        TheaterSceneRequest {
            cast_a: TheaterCastRef {
                role_id: "mumu".to_string(),
                name: "木木".to_string(),
            },
            cast_b: TheaterCastRef {
                role_id: "枫侵月".to_string(),
                name: "枫侵月".to_string(),
            },
            scene_id: "home".to_string(),
            base_beats: vec![
                TheaterScriptLine {
                    id: "b1".to_string(),
                    cast: "b".to_string(),
                    name: "枫侵月".to_string(),
                    text: "粥还要不要温一下？".to_string(),
                    stage_hint: None,
                    emotion: None,
                },
                TheaterScriptLine {
                    id: "b2".to_string(),
                    cast: "a".to_string(),
                    name: "木木".to_string(),
                    text: "哼。".to_string(),
                    stage_hint: None,
                    emotion: None,
                },
            ],
            applied_tweaks: vec![TheaterTweak {
                kind: "chip".to_string(),
                chip_label: Some("喝茶".to_string()),
                drama_seed: "苦药变笑料".to_string(),
                insert_after_beat_id: "b1".to_string(),
                lead_cast: "a".to_string(),
            }],
            fallback_beats: vec![
                TheaterScriptLine {
                    id: "b1".to_string(),
                    cast: "b".to_string(),
                    name: "枫侵月".to_string(),
                    text: "粥还要不要温一下？".to_string(),
                    stage_hint: None,
                    emotion: None,
                },
                TheaterScriptLine {
                    id: "p1".to_string(),
                    cast: "a".to_string(),
                    name: "木木".to_string(),
                    text: "补丁".to_string(),
                    stage_hint: None,
                    emotion: None,
                },
                TheaterScriptLine {
                    id: "b2".to_string(),
                    cast: "a".to_string(),
                    name: "木木".to_string(),
                    text: "哼。".to_string(),
                    stage_hint: None,
                    emotion: None,
                },
            ],
            max_beats: Some(10),
            mode: None,
            patch_variant: None,
            fork_templates: None,
            adapt_pass: None,
            poke_chips: None,
            pair_relation_id: None,
            pair_relation_hint: None,
            theater_scene: None,
            scene_brief: None,
            scene_setting_hint: None,
        }
    }

    #[test]
    fn resolve_ripple_context_splits_prefix() {
        let ctx = resolve_ripple_context(&sample_req());
        assert!(!ctx.full_rewrite);
        assert_eq!(ctx.prefix_beats.len(), 2);
        assert_eq!(ctx.prefix_beats[1].id, "p1");
        assert_eq!(ctx.ripple_skeleton.len(), 1);
        assert_eq!(ctx.ripple_skeleton[0].id, "b2");
    }

    #[test]
    fn merge_scene_beats_concatenates() {
        let prefix = vec![TheaterScriptLine {
            id: "b1".to_string(),
            cast: "b".to_string(),
            name: "枫侵月".to_string(),
            text: "前缀".to_string(),
            stage_hint: None,
            emotion: None,
        }];
        let ripple = vec![TheaterScriptLine {
            id: "r1".to_string(),
            cast: "a".to_string(),
            name: "木木".to_string(),
            text: "涟漪".to_string(),
            stage_hint: None,
            emotion: None,
        }];
        let merged = merge_scene_beats(&prefix, ripple);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].text, "前缀");
        assert_eq!(merged[1].text, "涟漪");
    }

    #[test]
    fn try_merge_ripple_rejects_id_conflict() {
        let ctx = resolve_ripple_context(&sample_req());
        let raw = r#"[{"id":"b1","cast":"b","name":"枫侵月","text":"冲突。"}]"#;
        assert!(try_merge_ripple(&ctx, raw, 10).is_none());
    }

    #[test]
    fn build_scene_prompt_includes_ripple_scope() {
        let req = sample_req();
        let ctx = resolve_ripple_context(&req);
        let p = build_scene_prompt(&req, &ctx, 10, false, "傲娇少女", "温柔管家");
        assert!(p.contains("木木"));
        assert!(p.contains("枫侵月"));
        assert!(p.contains("苦药变笑料"));
        assert!(p.contains("涟漪区"));
        assert!(p.contains("JSON 数组"));
        assert!(p.contains("人设摘要"));
        assert!(p.contains("傲娇少女"));
    }

    #[test]
    fn build_scene_prompt_strict_mode() {
        let req = sample_req();
        let ctx = resolve_ripple_context(&req);
        let p = build_scene_prompt(&req, &ctx, 10, true, "", "");
        assert!(p.contains("严格"));
    }

    #[test]
    fn build_scene_prompt_includes_drama_seed_persona_discipline() {
        let req = sample_req();
        let ctx = resolve_ripple_context(&req);
        let p = build_scene_prompt(&req, &ctx, 10, false, "傲娇", "温柔");
        assert!(p.contains("微调纪律"));
        assert!(p.contains("drama_seed"));
        assert!(p.contains("不得机械复制罐头 fork"));
        assert!(p.contains("勿照搬罐头 fork"));
    }

    #[test]
    fn truncate_chars_respects_limit() {
        let s = "一二三四五六七八九十";
        assert_eq!(truncate_chars(s, 5), "一二三四五…");
    }

    #[test]
    fn first_prompt_paragraph_skips_headers() {
        let text = "# Title\n\nFirst line.\nSecond line.";
        assert!(first_prompt_paragraph(text).contains("First line."));
    }

    #[test]
    fn beats_after_insert_returns_tail() {
        let req = sample_req();
        let tail = beats_after_insert(&req.base_beats, "b1");
        assert_eq!(tail.len(), 1);
        assert_eq!(tail[0].id, "b2");
    }

    #[test]
    fn parse_scene_json_plain_array() {
        let raw = r#"[{"id":"b1","cast":"b","name":"枫侵月","text":"你好。"}]"#;
        let beats = parse_scene_json(raw, 10).expect("parse");
        assert_eq!(beats.len(), 1);
        assert_eq!(beats[0].cast, "b");
    }

    #[test]
    fn parse_scene_json_code_fence() {
        let raw = "```json\n[{\"id\":\"b1\",\"cast\":\"a\",\"name\":\"木木\",\"text\":\"哼。\"}]\n```";
        let beats = parse_scene_json(raw, 10).expect("parse");
        assert_eq!(beats[0].name, "木木");
    }

    #[test]
    fn parse_scene_json_dirty_prefix() {
        let raw = "好的，这是剧本：\n[{\"id\":\"b1\",\"cast\":\"b\",\"name\":\"枫侵月\",\"text\":\"嗯。\"}]";
        let beats = parse_scene_json(raw, 10).expect("parse");
        assert_eq!(beats.len(), 1);
    }

    #[test]
    fn parse_scene_json_rejects_invalid_cast() {
        let raw = r#"[{"id":"b1","cast":"c","name":"X","text":"hi"}]"#;
        assert!(parse_scene_json(raw, 10).is_none());
    }

    #[test]
    fn parse_scene_json_rejects_empty_text() {
        let raw = r#"[{"id":"b1","cast":"a","name":"木木","text":"   "}]"#;
        assert!(parse_scene_json(raw, 10).is_none());
    }

    #[test]
    fn parse_scene_json_respects_max_beats() {
        let raw = r#"[{"id":"b1","cast":"a","name":"木木","text":"a"},{"id":"b2","cast":"b","name":"枫侵月","text":"b"}]"#;
        assert!(parse_scene_json(raw, 1).is_none());
    }

    #[test]
    fn strip_code_fence_no_fence() {
        assert_eq!(strip_code_fence("plain"), "plain");
    }

    #[test]
    fn parse_cast_adapt_json_object() {
        let raw = r#"{"beats":[{"id":"b1","cast":"b","name":"小枫","text":"适配开场。"}],"forks":[{"chip_id":"tea","patch_lines":[{"id":"tea-1","cast":"b","name":"小枫","text":"适配罐头。"}]}]}"#;
        let parsed = parse_cast_adapt_json(raw, 10).expect("parse");
        assert_eq!(parsed.beats.len(), 1);
        assert_eq!(parsed.forks.len(), 1);
        assert_eq!(parsed.forks[0].chip_id, "tea");
    }

    #[test]
    fn parse_cast_adapt_json_beats_only_without_forks() {
        let raw = r#"{"beats":[{"id":"b1","cast":"b","name":"诗梦","text":"……烦死了，粥自己不会热吗。"}]}"#;
        let parsed = parse_cast_adapt_json(raw, 10).expect("parse");
        assert_eq!(parsed.beats.len(), 1);
        assert!(parsed.forks.is_empty());
    }

    #[test]
    fn merge_adapted_beats_preserves_ids() {
        let skeleton = vec![
            TheaterScriptLine {
                id: "b1".to_string(),
                cast: "b".to_string(),
                name: "枫侵月".to_string(),
                text: "原文。".to_string(),
                stage_hint: None,
                emotion: None,
            },
            TheaterScriptLine {
                id: "b2".to_string(),
                cast: "a".to_string(),
                name: "木木".to_string(),
                text: "原文2。".to_string(),
                stage_hint: None,
                emotion: None,
            },
        ];
        let llm = vec![TheaterScriptLine {
            id: "b1".to_string(),
            cast: "b".to_string(),
            name: "小枫".to_string(),
            text: "改写。".to_string(),
            stage_hint: None,
            emotion: Some("happy".to_string()),
        }];
        let merged = merge_adapted_beats(&skeleton, &llm);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].text, "改写。");
        assert_eq!(merged[0].id, "b1");
        assert_eq!(merged[1].text, "原文2。");
    }

    #[test]
    fn merge_adapted_beats_rejects_cast_mismatch() {
        let skeleton = vec![TheaterScriptLine {
            id: "b1".to_string(),
            cast: "b".to_string(),
            name: "枫侵月".to_string(),
            text: "原文。".to_string(),
            stage_hint: None,
            emotion: None,
        }];
        let llm = vec![TheaterScriptLine {
            id: "b1".to_string(),
            cast: "a".to_string(),
            name: "木木".to_string(),
            text: "错 cast。".to_string(),
            stage_hint: None,
            emotion: None,
        }];
        let merged = merge_adapted_beats(&skeleton, &llm);
        assert_eq!(merged[0].text, "原文。");
    }

    #[test]
    fn merge_adapted_forks_by_chip_and_line_id() {
        let templates = vec![TheaterForkTemplate {
            chip_id: "tea".to_string(),
            insert_after_beat_id: "b6".to_string(),
            patch_lines: vec![TheaterScriptLine {
                id: "tea-1".to_string(),
                cast: "b".to_string(),
                name: "枫侵月".to_string(),
                text: "罐头原文。".to_string(),
                stage_hint: None,
                emotion: None,
            }],
        }];
        let raw = r#"{"beats":[{"id":"b1","cast":"b","name":"枫","text":"x"}],"forks":[{"chip_id":"tea","patch_lines":[{"id":"tea-1","cast":"b","name":"枫","text":"罐头改写。"}]}]}"#;
        let parsed = parse_cast_adapt_json(raw, 10).expect("parse");
        let merged = merge_adapted_forks(&templates, &parsed.forks);
        assert_eq!(merged[0].insert_after_beat_id, "b6");
        assert_eq!(merged[0].patch_lines[0].text, "罐头改写。");
    }

    #[test]
    fn build_cast_adapt_prompt_includes_persona() {
        let req = sample_req();
        let p = build_cast_adapt_prompt(&req, &[], 10, false, "傲娇", "温柔");
        assert!(p.contains("卡司适配"));
        assert!(p.contains("傲娇"));
        assert!(p.contains("JSON 对象"));
    }

    #[test]
    fn build_cast_adapt_prompt_pass_depth() {
        let mut req = sample_req();
        req.adapt_pass = Some("depth".to_string());
        let p = build_cast_adapt_prompt(&req, &[], 10, false, "傲娇", "温柔");
        assert!(p.contains("角色化大纲"));
    }

    #[test]
    fn parse_cast_rewrite_json_valid() {
        let raw = r#"{"beats":[
          {"id":"b1","cast":"b","name":"诗梦","text":"……烦死了。"},
          {"id":"b2","cast":"a","name":"木木","text":"哦。"},
          {"id":"b3","cast":"b","name":"诗梦","text":"快吃。"},
          {"id":"b4","cast":"a","name":"木木","text":"知道了。"},
          {"id":"b5","cast":"b","name":"诗梦","text":"要迟到了。"},
          {"id":"b6","cast":"a","name":"木木","text":"走吧。"}
        ],"forks":[{"chip_id":"tea","insert_after_beat_id":"b4","patch_lines":[
          {"id":"tea-1","cast":"b","name":"诗梦","text":"喝药。"},
          {"id":"tea-2","cast":"a","name":"木木","text":"不要！"}
        ]}]}"#;
        let parsed = parse_cast_rewrite_json(raw, 6, 12).expect("parse");
        assert_eq!(parsed.beats.len(), 6);
        assert_eq!(parsed.forks.len(), 1);
        assert_eq!(parsed.forks[0].insert_after_beat_id, "b4");
    }

    #[test]
    fn resolve_scene_mode_infers_cast_rewrite_without_mode_field() {
        let mut req = sample_req();
        req.mode = None;
        req.base_beats.clear();
        req.poke_chips = Some(vec![TheaterPokeChipDef {
            chip_id: "tea".to_string(),
            drama_seed: "苦药".to_string(),
            label: None,
        }]);
        assert_eq!(resolve_scene_mode(&req), Some("cast_rewrite"));
    }

    #[test]
    fn resolve_scene_mode_prefers_explicit_mode() {
        let mut req = sample_req();
        req.mode = Some("cast_rewrite".to_string());
        assert_eq!(resolve_scene_mode(&req), Some("cast_rewrite"));
    }

    #[test]
    fn build_cast_rewrite_prompt_includes_pair_relation() {
        let mut req = sample_req();
        req.base_beats.clear();
        req.pair_relation_id = Some("lover".to_string());
        req.pair_relation_hint = Some("恋人语气更软".to_string());
        let p = build_cast_rewrite_prompt(&req, 6, 12, false, "傲娇", "温柔");
        assert!(p.contains("双角色关系（lover）"));
        assert!(p.contains("恋人语气更软"));
    }

    #[test]
    fn build_cast_rewrite_prompt_supermarket_beats_only() {
        let mut req = sample_req();
        req.base_beats.clear();
        req.theater_scene = Some("supermarket".to_string());
        req.scene_brief = Some("超市采购：推购物车、抢特价。".to_string());
        req.scene_setting_hint = Some("地点限于超市卖场。".to_string());
        let p = build_cast_rewrite_prompt(&req, 6, 12, false, "傲娇", "温柔");
        assert!(p.contains("supermarket"));
        assert!(p.contains("超市采购"));
        assert!(p.contains("超市卖场"));
        assert!(p.contains("JSON 数组"));
        assert!(!p.contains("forks 须全部覆盖"));
    }

    #[test]
    fn build_cast_rewrite_prompt_breakfast_beats_only_even_with_poke_chips() {
        let mut req = sample_req();
        req.base_beats.clear();
        req.poke_chips = Some(vec![TheaterPokeChipDef {
            chip_id: "tea".to_string(),
            drama_seed: "苦药".to_string(),
            label: None,
        }]);
        let p = build_cast_rewrite_prompt(&req, 4, 12, false, "傲娇", "温柔");
        assert!(p.contains("JSON 数组"));
        assert!(p.contains("不要") && p.contains("forks"));
        assert!(!p.contains("forks 须全部覆盖"));
        assert!(!p.contains("patch_lines"));
        assert!(!p.contains("{theater_scene}"));
        assert!(!p.contains("stage_hint?"));
        assert!(!p.contains("emotion?"));
        assert!(p.contains("仅含 id、cast、text"));
    }

    #[test]
    fn cast_rewrite_requires_forks_when_poke_chips_present() {
        let mut req = sample_req();
        req.poke_chips = Some(vec![TheaterPokeChipDef {
            chip_id: "tea".to_string(),
            drama_seed: "苦药".to_string(),
            label: None,
        }]);
        assert!(cast_rewrite_requires_forks(&req));
        req.poke_chips = Some(vec![]);
        assert!(!cast_rewrite_requires_forks(&req));
    }

    #[test]
    fn parse_cast_rewrite_beats_loose_accepts_json_array() {
        let raw = r#"[
          {"id":"b1","cast":"b","text":"购物车在这边。"},
          {"id":"b2","cast":"a","text":"我不买。"},
          {"id":"b3","cast":"b","text":"特价鸡蛋。"},
          {"id":"b4","cast":"a","text":"知道了。"},
          {"id":"b5","cast":"b","text":"试吃区在那。"},
          {"id":"b6","cast":"a","text":"别塞给我。"}
        ]"#;
        let beats = parse_cast_rewrite_beats_loose(raw, 4, 12).expect("array parse");
        assert_eq!(beats.len(), 6);
        assert_eq!(beats[0].text, "购物车在这边。");
    }

    #[test]
    fn parse_cast_rewrite_beats_loose_ignores_invalid_forks() {
        let raw = r#"{"beats":[
          {"id":"b1","cast":"b","text":"第一句。"},
          {"id":"b2","cast":"a","text":"第二句。"},
          {"id":"b3","cast":"b","text":"第三句。"},
          {"id":"b4","cast":"a","text":"第四句。"}
        ],"forks":[{"chip_id":"bad","patch_lines":[]}]}"#;
        let beats = parse_cast_rewrite_beats_loose(raw, 4, 12).expect("beats despite bad forks");
        assert_eq!(beats.len(), 4);
    }

    #[test]
    fn try_parse_cast_rewrite_beats_only_normalizes_names() {
        let mut req = sample_req();
        req.base_beats.clear();
        req.cast_a.name = "木木".to_string();
        req.cast_b.name = "诗梦".to_string();
        let raw = r#"[{"cast":"b","text":"a"},{"cast":"a","text":"b"},{"cast":"b","text":"c"},{"cast":"a","text":"d"}]"#;
        let beats = try_parse_cast_rewrite_beats_only(&req, raw, 4, 12).expect("normalize");
        assert_eq!(beats[0].name, "诗梦");
        assert_eq!(beats[1].name, "木木");
    }

    #[test]
    fn parse_cast_rewrite_beats_loose_accepts_trailing_commas() {
        let raw = r#"[
          {"id":"b1","cast":"b","text":"第一句。"},
          {"id":"b2","cast":"a","text":"第二句。"},
          {"id":"b3","cast":"b","text":"第三句。"},
          {"id":"b4","cast":"a","text":"第四句。"},
        ]"#;
        let beats = parse_cast_rewrite_beats_loose(raw, 4, 12).expect("trailing comma");
        assert_eq!(beats.len(), 4);
    }

    #[test]
    fn try_parse_cast_rewrite_accepts_role_name_cast_and_dialogue_field() {
        let mut req = sample_req();
        req.cast_a.name = "木木".to_string();
        req.cast_b.name = "枫侵月".to_string();
        let raw = r#"说明如下：
[
  {"id":"b1","cast":"枫侵月","dialogue":"早。"},
  {"id":"b2","cast":"木木","dialogue":"哼。"},
  {"id":"b3","cast":"枫侵月","dialogue":"粥好了。"},
  {"id":"b4","cast":"木木","dialogue":"知道了。"}
]"#;
        let beats = try_parse_cast_rewrite_beats_only(&req, raw, 4, 12).expect("name cast");
        assert_eq!(beats.len(), 4);
        assert_eq!(beats[0].cast, "b");
        assert_eq!(beats[1].cast, "a");
    }

    #[test]
    fn salvage_rewrite_objects_accepts_malformed_array_wrapper() {
        let mut req = sample_req();
        req.cast_a.name = "木木".to_string();
        req.cast_b.name = "枫侵月".to_string();
        let raw = r#"[
          {"cast":"b","text":"一"},
          {"cast":"a","text":"二"},
          {"cast":"b","text":"三"},
          {"cast":"a","text":"四"},
        "#;
        let ctx = CastRewriteParseCtx::from_req(&req);
        let beats = salvage_rewrite_objects(raw, 4, 12, &ctx).expect("salvage");
        assert_eq!(beats.len(), 4);
    }

    #[test]
    fn build_cast_rewrite_minimal_prompt_is_short_and_fixed_count() {
        let req = sample_req();
        let p = build_cast_rewrite_minimal_prompt(&req, 8, "傲娇", "温柔");
        assert!(p.contains("恰好 8 条"));
        assert!(p.contains("从 [ 开始"));
        assert!(!p.contains("forks"));
    }
}

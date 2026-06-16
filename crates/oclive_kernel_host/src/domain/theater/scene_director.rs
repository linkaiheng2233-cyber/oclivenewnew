//! Theater scene director: one LLM call rewrites the ripple zone (after tweaks) as structured JSON beats.
//!
//! Bypasses `process_message` and six-slot orchestration; uses [`AppState::llm`] with the
//! effective model resolved from cast roles.

use crate::domain::effective_llm_model::resolve_effective_ollama_model;
use crate::domain::theater::scene_director_config::{ripple_max_beats, scene_llm_timeout_secs};
use crate::error::{AppError, Result};
use crate::models::dto::{
    TheaterForkTemplate, TheaterPokeChipDef, TheaterSceneRequest, TheaterSceneResponse,
    TheaterScriptLine,
};
use crate::state::AppState;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::time::Duration;

const MAX_BEAT_TEXT_LEN: usize = 500;
const MAX_STAGE_HINT_LEN: usize = 120;

fn scene_response(
    beats: Vec<TheaterScriptLine>,
    source: &str,
    model: String,
    adapted_forks: Option<Vec<TheaterForkTemplate>>,
) -> TheaterSceneResponse {
    scene_response_with_meta(beats, source, model, adapted_forks, None, None)
}

fn scene_response_with_meta(
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
struct RippleContext {
    prefix_beats: Vec<TheaterScriptLine>,
    ripple_skeleton: Vec<TheaterScriptLine>,
    full_rewrite: bool,
}

/// Resolve theater scene mode; infer `cast_rewrite` when optional `mode` was dropped by an older kernel DTO.
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

    let ctx = resolve_ripple_context(req);
    let max_beats = req
        .max_beats
        .unwrap_or_else(ripple_max_beats)
        .clamp(4, 64);
    let model = resolve_scene_model(state, req).await?;
    let source = resolve_llm_source_label(state);
    let timeout = Duration::from_secs(scene_llm_timeout_secs());

    let persona_a = resolve_cast_persona(state, req.cast_a.role_id.as_str()).await;
    let persona_b = resolve_cast_persona(state, req.cast_b.role_id.as_str()).await;

    let prompt = build_scene_prompt(req, &ctx, max_beats, false, persona_a.as_str(), persona_b.as_str());
    let raw = match tokio::time::timeout(timeout, state.llm.generate_tag(model.as_str(), prompt.as_str())).await
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

    let strict_prompt = build_scene_prompt(req, &ctx, max_beats, true, persona_a.as_str(), persona_b.as_str());
    let retry_raw = match tokio::time::timeout(
        timeout,
        state
            .llm
            .generate_tag(model.as_str(), strict_prompt.as_str()),
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
    let model = resolve_scene_model(state, req).await?;
    let source = resolve_llm_source_label(state);
    let timeout = Duration::from_secs(scene_llm_timeout_secs());
    let templates = req.fork_templates.clone().unwrap_or_default();

    let persona_a = resolve_cast_persona(state, req.cast_a.role_id.as_str()).await;
    let persona_b = resolve_cast_persona(state, req.cast_b.role_id.as_str()).await;

    let prompt = build_cast_adapt_prompt(req, &templates, max_beats, false, persona_a.as_str(), persona_b.as_str());
    let raw = match tokio::time::timeout(timeout, state.llm.generate_tag(model.as_str(), prompt.as_str())).await
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

    let strict_prompt =
        build_cast_adapt_prompt(req, &templates, max_beats, true, persona_a.as_str(), persona_b.as_str());
    let retry_raw = match tokio::time::timeout(
        timeout,
        state
            .llm
            .generate_tag(model.as_str(), strict_prompt.as_str()),
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
    let min_beats = 6_u32;
    let max_beats = req
        .max_beats
        .unwrap_or(12)
        .clamp(min_beats, 16);
    let model = resolve_scene_model(state, req).await?;
    let source = resolve_llm_source_label(state);
    let timeout = Duration::from_secs(scene_llm_timeout_secs());
    let poke_chips = req.poke_chips.clone().unwrap_or_default();

    let persona_a = resolve_cast_persona(state, req.cast_a.role_id.as_str()).await;
    let persona_b = resolve_cast_persona(state, req.cast_b.role_id.as_str()).await;

    let prompt = build_cast_rewrite_prompt(
        req,
        &poke_chips,
        min_beats,
        max_beats,
        false,
        persona_a.as_str(),
        persona_b.as_str(),
    );
    let raw = match tokio::time::timeout(timeout, state.llm.generate_tag(model.as_str(), prompt.as_str())).await
    {
        Ok(Ok(t)) => t,
        Ok(Err(e)) => {
            tracing::warn!(target: "oclive_theater", "cast_rewrite LLM failed: {e}");
            return Ok(cast_rewrite_fallback_response(req, &model, "fallback", "rewrite_llm_error"));
        }
        Err(_) => {
            tracing::warn!(target: "oclive_theater", "cast_rewrite LLM timed out ({}s)", scene_llm_timeout_secs());
            return Ok(cast_rewrite_fallback_response(req, &model, "fallback", "rewrite_llm_timeout"));
        }
    };

    if let Some((beats, forks)) = try_parse_cast_rewrite(req, &poke_chips, &raw, min_beats, max_beats) {
        return Ok(scene_response(beats, source, model.clone(), Some(forks)));
    }

    if let Some(beats) = try_parse_cast_rewrite_beats_only(req, &raw, min_beats, max_beats) {
        tracing::info!(
            target: "oclive_theater",
            "cast_rewrite accepted beats-only (fork parse incomplete)"
        );
        return Ok(cast_rewrite_beats_only_response(req, beats, &model, source));
    }

    let strict_prompt = build_cast_rewrite_prompt(
        req,
        &poke_chips,
        min_beats,
        max_beats,
        true,
        persona_a.as_str(),
        persona_b.as_str(),
    );
    let retry_raw = match tokio::time::timeout(
        timeout,
        state
            .llm
            .generate_tag(model.as_str(), strict_prompt.as_str()),
    )
    .await
    {
        Ok(Ok(t)) => t,
        Ok(Err(e)) => {
            tracing::warn!(target: "oclive_theater", "cast_rewrite retry LLM failed: {e}");
            return Ok(cast_rewrite_fallback_response(req, &model, "fallback", "rewrite_llm_error"));
        }
        Err(_) => {
            tracing::warn!(target: "oclive_theater", "cast_rewrite retry LLM timed out");
            return Ok(cast_rewrite_fallback_response(req, &model, "fallback", "rewrite_llm_timeout"));
        }
    };

    if let Some((beats, forks)) = try_parse_cast_rewrite(req, &poke_chips, &retry_raw, min_beats, max_beats) {
        return Ok(scene_response(beats, source, model, Some(forks)));
    }

    if let Some(beats) = try_parse_cast_rewrite_beats_only(req, &retry_raw, min_beats, max_beats) {
        tracing::info!(
            target: "oclive_theater",
            "cast_rewrite retry accepted beats-only (fork parse incomplete)"
        );
        return Ok(cast_rewrite_beats_only_response(req, beats, &model, source));
    }

    tracing::warn!(target: "oclive_theater", "cast_rewrite parse failed; using fallback");
    Ok(cast_rewrite_fallback_response(req, &model, "fallback", "rewrite_parse_failed"))
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
    let parsed = parse_cast_rewrite_json(raw, min_beats, max_beats)?;
    normalize_rewrite_beats(req, &parsed.beats)
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

fn build_cast_rewrite_prompt(
    req: &TheaterSceneRequest,
    poke_chips: &[TheaterPokeChipDef],
    min_beats: u32,
    max_beats: u32,
    strict: bool,
    persona_a: &str,
    persona_b: &str,
) -> String {
    let chips_json = serde_json::to_string(poke_chips).unwrap_or_else(|_| "[]".to_string());
    let strict_tail = if strict {
        "\n【严格】只输出 JSON 对象，无 Markdown、无解释。forks 必须包含全部 chip_id。"
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

    format!(
        r#"卡司重写：为以下两位角色**从零**撰写「早饭 · 上学前」双人短剧。不要沿用任何现成台词或剧情模板，须完全贴合人设关系与说话方式。

cast a={name_a}，cast b={name_b}，场景={scene_id}。仅 a/b 两人发言，禁止第三人。

人设摘要：
{persona_block}

戳点分支（forks 须全部覆盖，每项含 chip_id、insert_after_beat_id、patch_lines）：
{chips_json}

撰写要求：
1. beats：{min_beats}-{max_beats} 条，id 依次为 b1,b2,b3…（连续）；cast 仅 a/b；name 用对应显示名；text 非空≤{max_text}字；写**全新**对白与小事件（仍可落在厨房/餐桌/玄关出门，但不要复制常见「温粥/天气预报/书包」流水账，除非符合该人设）。
2. forks：每个 chip_id 一条；insert_after_beat_id 必须是 beats 中某条 id（建议中后段）；patch_lines 3-4 条，id 自定如 tea-1；体现 drama_seed 意图。
3. 交替发言、有戏感、口语自然中文。

输出契约：{{"beats":[{{"id","cast","name","text","stage_hint?","emotion?"}}],"forks":[{{"chip_id","insert_after_beat_id","patch_lines":[...]}}]}}

示例 beat：{{"id":"b1","cast":"b","name":"{name_b}","text":"……","emotion":"happy"}}{strict_tail}
"#,
        name_a = req.cast_a.name.trim(),
        name_b = req.cast_b.name.trim(),
        scene_id = req.scene_id.trim(),
        max_text = MAX_BEAT_TEXT_LEN,
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
fn build_cast_adapt_prompt(
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

    format!(
        r#"卡司适配：双人早饭上学前剧场。cast a={name_a}，cast b={name_b}，场景={scene_id}。仅 a/b 发言。
{pass_block}
{persona_block}
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
        max_text = MAX_BEAT_TEXT_LEN,
    )
}

fn cast_adapt_pass_instructions(pass: Option<&str>) -> String {
    let focus = match pass.map(str::trim).filter(|s| !s.is_empty()) {
        Some("voice") => {
            "【本轮·语气人设】第一轮：在保持事件顺序与 beat id 不变的前提下，把每位角色的台词改成其人设口吻；同步调整 emotion/stage_hint 以贴合性格（毒舌/温柔/别扭等）。禁止只改姓名。"
        }
        Some("depth") => {
            "【本轮·角色化大纲】第二轮：在早饭→上学前的时间框架内，进一步改写台词内容与 stage_hint，使互动、拌嘴方式、关心/抵触的表达方式更符合两位角色的关系与性格；可调整具体物件与情绪转折，但 beat id/cast 不可变，仍须落在同一早餐场景。"
        }
        Some("polish") => {
            "【本轮·戳点收束】第三轮：重点改写 forks 戳点罐头台词；beats 做最终通顺与人设一致性润色，确保全剧台词风格统一、角色区分度明显。"
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

async fn resolve_scene_model(state: &AppState, req: &TheaterSceneRequest) -> Result<String> {
    let role_id = req.cast_a.role_id.trim();
    let session_ns = format!("theater:{}", req.scene_id.trim());
    if let Ok(role) = state.load_role_cached_async(role_id).await {
        return resolve_effective_ollama_model(state, role.as_ref(), session_ns.as_str()).await;
    }
    if let Ok(role) = state.load_role_cached_async(req.cast_b.role_id.trim()).await {
        return resolve_effective_ollama_model(state, role.as_ref(), session_ns.as_str()).await;
    }
    Ok(state.ollama_model.clone())
}

fn resolve_llm_source_label(state: &AppState) -> &'static str {
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
async fn resolve_cast_persona(state: &AppState, role_id: &str) -> String {
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
fn beats_after_insert(
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
fn merge_scene_beats(
    prefix: &[TheaterScriptLine],
    ripple: Vec<TheaterScriptLine>,
) -> Vec<TheaterScriptLine> {
    let mut out = prefix.to_vec();
    out.extend(ripple);
    out
}

fn ripple_ids_conflict(prefix: &[TheaterScriptLine], ripple: &[TheaterScriptLine]) -> bool {
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
fn build_scene_prompt(
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
        format!("微调意图：{tweaks_json}")
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

    format!(
        r#"场景导演：双人早饭剧场。cast a={name_a}，cast b={name_b}，场景={scene_id}。仅 a/b 发言。
{persona_block}
{scope_block}

{tweak_block}

输出契约：JSON 数组，每元素 {{"id","cast":"a"|"b","name","text","stage_hint?","emotion?"}}。
规则：只输出{output_scope}；总拍数≤{max_beats}；text 非空≤{max_text}字；name 与 cast 一致；台词须符合各人设；禁止脱离早饭上学前场景；不得新增第三人。

示例：[{{"id":"r1","cast":"b","name":"枫侵月","text":"粥还要不要温一下？","stage_hint":"推碗","emotion":"happy"}},{{"id":"r2","cast":"a","name":"木木","text":"……谁要你温了。","emotion":"shy"}}]{strict_tail}
"#,
        name_a = req.cast_a.name.trim(),
        name_b = req.cast_b.name.trim(),
        scene_id = req.scene_id.trim(),
        output_scope = if ctx.full_rewrite {
            "整场"
        } else {
            "涟漪区（不含前缀）"
        },
        max_text = MAX_BEAT_TEXT_LEN,
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
            fork_templates: None,
            adapt_pass: None,
            poke_chips: None,
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
}

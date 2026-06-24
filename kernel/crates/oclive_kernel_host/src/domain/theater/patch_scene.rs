//! Poke patch mode: local prose micro-scene inserted after anchor; official skeleton tail preserved.

use crate::domain::theater::drama_guardrails::{self, PATCH_TITLE};
use crate::domain::theater::scene_director::{
    beats_after_insert, resolve_cast_persona, resolve_llm_source_label, resolve_theater_llm,
    ripple_ids_conflict, scene_response, scene_response_with_meta,
};
use crate::domain::theater::scene_director_config::{
    patch_max_lines, patch_partner_reply_enabled, scene_llm_timeout_secs,
};
use crate::error::Result;
use crate::models::dto::{
    TheaterSceneRequest, TheaterSceneResponse, TheaterScriptLine, TheaterTweak,
};
use crate::state::AppState;
use std::time::Duration;

const MAX_PATCH_TEXT_LEN: usize = 500;
const MAX_PATCH_HINT_LEN: usize = 120;

pub(crate) struct PatchContext {
    pub(crate) prefix_beats: Vec<TheaterScriptLine>,
    pub(crate) skeleton_tail: Vec<TheaterScriptLine>,
    pub(crate) canned_patch: Vec<TheaterScriptLine>,
    pub(crate) tweak: TheaterTweak,
    pub(crate) chip_slug: String,
}

/// Generate a poke patch scene: prefix + LLM prose patch + immutable skeleton tail.
///
/// # Errors
///
/// Returns `AppError` when patch context is invalid or the director LLM call fails.
pub async fn generate_patch_scene(
    state: &AppState,
    req: &TheaterSceneRequest,
) -> Result<TheaterSceneResponse> {
    let Some(ctx) = resolve_patch_context(req) else {
        return Ok(patch_fallback_response(
            req,
            state.ollama_model.as_str(),
            "fallback",
            "patch_no_tweak",
        ));
    };

    let max_lines = patch_max_lines() as usize;
    let variant_index = req.patch_variant.unwrap_or(0);
    let (llm, model) = match resolve_theater_llm(state, req).await {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(target: "oclive_theater", "patch resolve_theater_llm failed: {e}");
            return Ok(patch_merge_fallback(
                &ctx,
                req,
                state.ollama_model.as_str(),
                "fallback",
            ));
        }
    };
    let source = resolve_llm_source_label(state);
    let timeout = Duration::from_secs(scene_llm_timeout_secs());

    let persona_a = resolve_cast_persona(state, req.cast_a.role_id.as_str()).await;
    let persona_b = resolve_cast_persona(state, req.cast_b.role_id.as_str()).await;

    let prompt = crate::domain::theater_director::build_theater_prompt(
        state,
        &crate::domain::theater_director::patch_prompt_input(
            req,
            &ctx,
            max_lines,
            false,
            persona_a.as_str(),
            persona_b.as_str(),
            variant_index,
        ),
    );
    let raw =
        match tokio::time::timeout(timeout, llm.generate(model.as_str(), prompt.as_str())).await {
            Ok(Ok(t)) => t,
            Ok(Err(e)) => {
                tracing::warn!(target: "oclive_theater", "patch LLM failed: {e}");
                return Ok(patch_merge_fallback(&ctx, req, &model, "fallback"));
            }
            Err(_) => {
                tracing::warn!(
                    target: "oclive_theater",
                    "patch LLM timed out ({}s)",
                    scene_llm_timeout_secs()
                );
                return Ok(patch_merge_fallback(&ctx, req, &model, "fallback"));
            }
        };

    if let Some(beats) = try_merge_patch(&ctx, req, &raw, max_lines) {
        return Ok(scene_response(beats, source, model.clone(), None));
    }

    let strict_prompt = crate::domain::theater_director::build_theater_prompt(
        state,
        &crate::domain::theater_director::patch_prompt_input(
            req,
            &ctx,
            max_lines,
            true,
            persona_a.as_str(),
            persona_b.as_str(),
            variant_index,
        ),
    );
    let retry_raw = match tokio::time::timeout(
        timeout,
        llm.generate(model.as_str(), strict_prompt.as_str()),
    )
    .await
    {
        Ok(Ok(t)) => t,
        Ok(Err(e)) => {
            tracing::warn!(target: "oclive_theater", "patch retry LLM failed: {e}");
            return Ok(patch_merge_fallback(&ctx, req, &model, "fallback"));
        }
        Err(_) => {
            tracing::warn!(target: "oclive_theater", "patch retry LLM timed out");
            return Ok(patch_merge_fallback(&ctx, req, &model, "fallback"));
        }
    };

    if let Some(beats) = try_merge_patch(&ctx, req, &retry_raw, max_lines) {
        return Ok(scene_response(beats, source, model, None));
    }

    tracing::warn!(target: "oclive_theater", "patch parse failed; using canned fork");
    Ok(patch_merge_fallback(&ctx, req, &model, "fallback"))
}

fn patch_merge_fallback(
    ctx: &PatchContext,
    _req: &TheaterSceneRequest,
    model: &str,
    source: &str,
) -> TheaterSceneResponse {
    let beats = merge_patch_beats(
        &ctx.prefix_beats,
        ctx.canned_patch.clone(),
        &ctx.skeleton_tail,
    );
    scene_response_with_meta(
        beats,
        source,
        model.to_string(),
        None,
        Some("patch_parse_failed"),
        None,
    )
}

fn patch_fallback_response(
    req: &TheaterSceneRequest,
    model: &str,
    source: &str,
    reason: &str,
) -> TheaterSceneResponse {
    scene_response_with_meta(
        req.fallback_beats.clone(),
        source,
        model.to_string(),
        None,
        Some(reason),
        None,
    )
}

fn resolve_patch_context(req: &TheaterSceneRequest) -> Option<PatchContext> {
    let tweak = req.applied_tweaks.last()?.clone();
    let anchor = tweak.insert_after_beat_id.trim();
    if anchor.is_empty() {
        return None;
    }

    let skeleton_tail = beats_after_insert(&req.base_beats, anchor);
    let anchor_idx = req.fallback_beats.iter().position(|b| b.id == anchor)?;
    let prefix_beats = req.fallback_beats[..=anchor_idx].to_vec();
    let canned_patch = extract_canned_patch(&req.fallback_beats, anchor, &skeleton_tail);
    let chip_slug = infer_chip_slug(&tweak, &canned_patch);

    Some(PatchContext {
        prefix_beats,
        skeleton_tail,
        canned_patch,
        tweak,
        chip_slug,
    })
}

fn extract_canned_patch(
    fallback: &[TheaterScriptLine],
    anchor: &str,
    skeleton_tail: &[TheaterScriptLine],
) -> Vec<TheaterScriptLine> {
    let Some(anchor_idx) = fallback.iter().position(|b| b.id == anchor) else {
        return Vec::new();
    };
    let start = anchor_idx + 1;
    if start >= fallback.len() {
        return Vec::new();
    }
    if skeleton_tail.is_empty() {
        return fallback[start..].to_vec();
    }
    let tail_id = skeleton_tail[0].id.as_str();
    let Some(tail_idx) = fallback.iter().position(|b| b.id == tail_id) else {
        return fallback[start..].to_vec();
    };
    if tail_idx <= start {
        return Vec::new();
    }
    fallback[start..tail_idx].to_vec()
}

fn infer_chip_slug(tweak: &TheaterTweak, canned: &[TheaterScriptLine]) -> String {
    if let Some(first) = canned.first() {
        if let Some((prefix, _)) = first.id.split_once('-') {
            if !prefix.is_empty() {
                return prefix.to_string();
            }
        }
    }
    if tweak.kind == "custom" {
        return "custom".to_string();
    }
    "poke".to_string()
}

fn normalize_lead_cast(lead: &str) -> &str {
    if lead.eq_ignore_ascii_case("b") {
        "b"
    } else {
        "a"
    }
}

fn lead_speaker(req: &TheaterSceneRequest, lead_cast: &str) -> (String, String) {
    if lead_cast == "b" {
        (
            req.cast_b.name.trim().to_string(),
            req.cast_a.name.trim().to_string(),
        )
    } else {
        (
            req.cast_a.name.trim().to_string(),
            req.cast_b.name.trim().to_string(),
        )
    }
}

fn merge_patch_beats(
    prefix: &[TheaterScriptLine],
    patch: Vec<TheaterScriptLine>,
    skeleton_tail: &[TheaterScriptLine],
) -> Vec<TheaterScriptLine> {
    let mut out = prefix.to_vec();
    out.extend(patch);
    out.extend(skeleton_tail.iter().cloned());
    out
}

fn try_merge_patch(
    ctx: &PatchContext,
    req: &TheaterSceneRequest,
    raw: &str,
    max_lines: usize,
) -> Option<Vec<TheaterScriptLine>> {
    let lead = normalize_lead_cast(ctx.tweak.lead_cast.as_str());
    let patch = parse_patch_prose(raw, req, lead, ctx.chip_slug.as_str(), max_lines)?;
    if patch.is_empty() {
        return None;
    }
    if ripple_ids_conflict(&ctx.prefix_beats, &patch) {
        return None;
    }
    Some(merge_patch_beats(
        &ctx.prefix_beats,
        patch,
        &ctx.skeleton_tail,
    ))
}

#[must_use]
pub(crate) fn build_patch_prompt(
    req: &TheaterSceneRequest,
    ctx: &PatchContext,
    max_lines: usize,
    strict: bool,
    persona_a: &str,
    persona_b: &str,
    variant_index: u8,
) -> String {
    let lead = normalize_lead_cast(ctx.tweak.lead_cast.as_str());
    let (speaker_name, partner_name) = lead_speaker(req, lead);

    let chip_label = ctx
        .tweak
        .chip_label
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or("剧情转折");
    let drama_seed = ctx.tweak.drama_seed.trim();

    let context_lines: Vec<String> = ctx
        .prefix_beats
        .iter()
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(|b| format!("{}：{}", b.name.trim(), b.text.trim()))
        .collect();
    let context = context_lines.join("\n");

    let style_examples = build_style_examples_block(&ctx.prefix_beats, &ctx.canned_patch);

    let mut parts: Vec<String> = Vec::new();
    parts.push(PATCH_TITLE.to_string());
    parts.push(format!(
        "这是一幕双人日常戏。本事件主角是「{speaker_name}」（cast {lead}），须承担事件主体与第一反应；对手戏是「{partner_name}」。"
    ));
    parts.push(format!("观众刚刚按下了剧情转折「{chip_label}」。"));
    if !drama_seed.is_empty() {
        parts.push(format!("本场戏剧目标：{drama_seed}"));
    }
    parts.push(drama_guardrails::drama_guardrails_full(
        req.theater_scene.as_deref(),
    ));

    if !persona_a.is_empty() || !persona_b.is_empty() {
        parts.push(String::new());
        parts.push("【人设摘要 · 必须贴合】".to_string());
        if !persona_a.is_empty() {
            parts.push(format!(
                "{}（cast a）：{}",
                req.cast_a.name.trim(),
                persona_a
            ));
        }
        if !persona_b.is_empty() {
            parts.push(format!(
                "{}（cast b）：{}",
                req.cast_b.name.trim(),
                persona_b
            ));
        }
    }

    parts.push(String::new());
    parts.push("【演出要求】".to_string());
    parts.push(format!(
        "· 写出「{speaker_name}」接下来的 1–{max_lines} 句台词，每句一行，格式：角色名：台词"
    ));
    if patch_partner_reply_enabled() {
        parts.push(format!(
            "· 「{partner_name}」若回句，须带与主角形成性格反差的反应（吐槽/关心/嘴硬/害羞等），禁止礼貌式「好的」「没事吧」敷衍接话；最多回一句，仍算在上述句数内"
        ));
    }
    parts.push("· 至少一句带上动作或神态，单独成行用括号包住，例：(耳朵红了)".to_string());
    parts.push("· 紧接上文语气，口语化、贴合人设；不要旁白、解说、JSON 或引号".to_string());
    parts.push("· 总字数 100 字以内".to_string());
    parts.push(format!(
        "· 禁止把本事件安到「{partner_name}」身上；主角必须是「{speaker_name}」"
    ));

    if variant_index == 1 {
        parts.push(
            "· 这是第二版候选：同一事件的不同性格演绎——换情绪走向与措辞，勿换词复述第一版，勿重复同一情节节拍".to_string(),
        );
    }

    if strict {
        parts.push("· 【严格模式】只输出对白行与括号动作行，不要任何前缀说明".to_string());
    }

    parts.push(String::new());
    parts.push("【刚刚发生的对白】".to_string());
    parts.push(if context.is_empty() {
        "（无）".to_string()
    } else {
        context
    });

    parts.push(String::new());
    parts.push("【可参考的情绪走向（仅作灵感，请改写出新意，禁止照抄）】".to_string());
    parts.push(if style_examples.is_empty() {
        "（自由发挥）".to_string()
    } else {
        style_examples
    });

    parts.join("\n")
}

fn build_style_examples_block(
    prefix: &[TheaterScriptLine],
    canned: &[TheaterScriptLine],
) -> String {
    let mut lines: Vec<String> = Vec::new();
    for b in prefix
        .iter()
        .rev()
        .take(2)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
    {
        lines.push(format!("{}：{}", b.name.trim(), b.text.trim()));
    }
    if let Some(first) = canned.first() {
        lines.push(format!("{}：{}", first.name.trim(), first.text.trim()));
    }
    lines.join("\n")
}

#[must_use]
pub fn parse_patch_prose(
    raw: &str,
    req: &TheaterSceneRequest,
    lead_cast: &str,
    chip_slug: &str,
    max_lines: usize,
) -> Option<Vec<TheaterScriptLine>> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    let fallback_name = if lead_cast == "b" {
        req.cast_b.name.trim()
    } else {
        req.cast_a.name.trim()
    };

    let mut lines: Vec<TheaterScriptLine> = Vec::new();
    let id_prefix = format!("patch-{chip_slug}");

    for row in trimmed.split('\n').map(str::trim).filter(|s| !s.is_empty()) {
        if lines.len() >= max_lines {
            break;
        }
        if row.starts_with('(') && row.ends_with(')') {
            if let Some(last) = lines.last_mut() {
                last.stage_hint = Some(
                    row.trim_start_matches('(')
                        .trim_end_matches(')')
                        .chars()
                        .take(MAX_PATCH_HINT_LEN)
                        .collect(),
                );
            }
            continue;
        }
        let (name, text) =
            if let Some((n, t)) = row.split_once('：').or_else(|| row.split_once(':')) {
                (n.trim(), t.trim())
            } else {
                (fallback_name, row)
            };
        if text.is_empty() {
            continue;
        }
        let cast = resolve_name_to_cast(name, req, lead_cast);
        let line_id = format!("{}-{}", id_prefix, lines.len());
        lines.push(TheaterScriptLine {
            id: line_id,
            cast: cast.to_string(),
            name: name.to_string(),
            text: text.chars().take(MAX_PATCH_TEXT_LEN).collect(),
            stage_hint: None,
            emotion: None,
        });
    }

    if lines.is_empty() {
        None
    } else {
        Some(lines)
    }
}

fn resolve_name_to_cast<'a>(name: &str, req: &TheaterSceneRequest, lead_cast: &'a str) -> &'a str {
    let na = req.cast_a.name.trim();
    let nb = req.cast_b.name.trim();
    if name == na || (!na.is_empty() && name.contains(na)) {
        "a"
    } else if name == nb || (!nb.is_empty() && name.contains(nb)) {
        "b"
    } else {
        lead_cast
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::dto::{TheaterCastRef, TheaterTweak};

    fn sample_patch_req() -> TheaterSceneRequest {
        TheaterSceneRequest {
            cast_a: TheaterCastRef {
                role_id: "mumu".to_string(),
                name: "木木".to_string(),
            },
            cast_b: TheaterCastRef {
                role_id: "feng".to_string(),
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
                TheaterScriptLine {
                    id: "b3".to_string(),
                    cast: "b".to_string(),
                    name: "枫侵月".to_string(),
                    text: "官方尾部。".to_string(),
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
                    id: "tea-1".to_string(),
                    cast: "b".to_string(),
                    name: "枫侵月".to_string(),
                    text: "罐头补丁".to_string(),
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
                TheaterScriptLine {
                    id: "b3".to_string(),
                    cast: "b".to_string(),
                    name: "枫侵月".to_string(),
                    text: "官方尾部。".to_string(),
                    stage_hint: None,
                    emotion: None,
                },
            ],
            max_beats: None,
            mode: Some("patch".to_string()),
            patch_variant: None,
            fork_templates: None,
            adapt_pass: None,
            poke_chips: None,
            pair_relation_id: None,
            pair_relation_hint: None,
            theater_scene: None,
            scene_brief: None,
            scene_setting_hint: None,
            script_outline: None,
        }
    }

    #[test]
    fn resolve_patch_context_splits_prefix_and_tail() {
        let ctx = resolve_patch_context(&sample_patch_req()).expect("ctx");
        assert_eq!(ctx.prefix_beats.len(), 1);
        assert_eq!(ctx.prefix_beats[0].id, "b1");
        assert_eq!(ctx.canned_patch.len(), 1);
        assert_eq!(ctx.canned_patch[0].id, "tea-1");
        assert_eq!(ctx.skeleton_tail.len(), 2);
        assert_eq!(ctx.skeleton_tail[0].id, "b2");
        assert_eq!(ctx.chip_slug, "tea");
    }

    #[test]
    fn parse_patch_prose_assigns_cast_and_stage_hint() {
        let req = sample_patch_req();
        let raw = "木木：这味道像惩罚！\n(整张脸皱成一团)\n枫侵月：乖，一口。";
        let lines = parse_patch_prose(raw, &req, "a", "tea", 4).expect("lines");
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].cast, "a");
        assert_eq!(lines[0].id, "patch-tea-0");
        assert_eq!(lines[0].stage_hint.as_deref(), Some("整张脸皱成一团"));
        assert_eq!(lines[1].cast, "b");
    }

    #[test]
    fn try_merge_patch_preserves_skeleton_tail() {
        let req = sample_patch_req();
        let ctx = resolve_patch_context(&req).expect("ctx");
        let raw = "木木：补丁台词。";
        let merged = try_merge_patch(&ctx, &req, raw, 4).expect("merged");
        assert_eq!(merged.len(), 4);
        assert_eq!(merged[0].id, "b1");
        assert_eq!(merged[1].id, "patch-tea-0");
        assert_eq!(merged[2].id, "b2");
        assert_eq!(merged[3].id, "b3");
        assert_eq!(merged[3].text, "官方尾部。");
    }

    #[test]
    fn build_patch_prompt_includes_lead_and_variant() {
        let req = sample_patch_req();
        let ctx = resolve_patch_context(&req).expect("ctx");
        let p = build_patch_prompt(&req, &ctx, 3, false, "傲娇", "温柔", 1);
        assert!(p.contains("木木"));
        assert!(p.contains(PATCH_TITLE));
        assert!(p.contains(drama_guardrails::GUARDRAILS_HEADER));
        assert!(p.contains("第二版候选"));
        assert!(p.contains("苦药变笑料"));
        assert!(p.contains("性格反差"));
    }
}

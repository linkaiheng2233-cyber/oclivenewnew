//! Chat Pro adult-extension prompt and structured reply adapter.
//!
//! This module is deliberately a narrow adapter: the universal turn pipeline
//! remains authoritative, while an opted-in Chat Pro request may add one prompt
//! section and parse one optional response envelope.

use crate::models::dto::{
    AdultBeatDto, AdultInteractionAction, AdultInteractionRequest, AdultInteractionState,
};
use crate::models::{AdultRoleExtension, Role};
use serde::Deserialize;

const ADULT_OUTPUT_TITLE: &str = "Chat Pro 成人扩展（结构化输出）";
const ADULT_OUTPUT_BOUNDARY: &str = r#"【本轮最终输出契约】本段替代上文的普通“只输出角色台词”边界。只输出一个合法 JSON 对象，不要代码围栏、前后说明或额外文本。四个字段都必须存在：
{"dialogue":"角色本人说的话","narration":"仅角色动作、环境与过程描写；没有时填空字符串","interaction_state":"active","next_beat_interval_ms":4000}
dialogue 不得代写用户；narration 不得断言用户未表达的动作、选择、感受或身体反应；interaction_state 只能是 inactive、active、ended；next_beat_interval_ms 必须是正整数。"#;

#[derive(Debug, Deserialize)]
struct AdultBeatWire {
    dialogue: String,
    #[serde(default)]
    narration: String,
    #[serde(default)]
    interaction_state: AdultInteractionState,
    #[serde(default)]
    next_beat_interval_ms: Option<u64>,
}

fn action_label(action: AdultInteractionAction) -> &'static str {
    match action {
        AdultInteractionAction::Message => "message",
        AdultInteractionAction::Continue => "continue",
        AdultInteractionAction::Exit => "exit",
    }
}

fn scene_direction(extension: &AdultRoleExtension, scene_id: &str) -> String {
    let Some(scene) = extension.scenes.get(scene_id) else {
        return "当前场景没有创作者专用走向；仅依据基础场景、人设和用户输入自然回应。".to_string();
    };
    format!(
        "当前场景走向：{}\n动作连续性：{}\n场景对话建议：{}",
        scene.direction.trim(),
        scene.action_flow.trim(),
        scene.dialogue_guidance.trim()
    )
}

/// Build the gated prompt body. `None` means the universal base path is used.
#[must_use]
pub fn prompt_section(
    role: &Role,
    scene_id: &str,
    request: Option<&AdultInteractionRequest>,
    identity_allows_adult: bool,
) -> Option<String> {
    let extension = role.adult_extension.as_ref()?;
    let request = request?;
    if !request.gates_open() || !identity_allows_adult {
        return None;
    }

    Some(format!(
        r#"角色包声明：角色为成年人。当前请求已经通过 Chat Pro 的成年确认、全局开关和角色开关。
成人状态人设增量：
{persona}

调情与亲密对话指导：
{dialogue_guidance}

{scene_direction}

当前互动状态：{active}
本轮动作：{action}
创作者节奏模式：{pacing_mode}
创作者建议间隔（毫秒）：{interval}

输出规则：
1. 只输出一个 JSON 对象，字段必须是 dialogue、narration、interaction_state、next_beat_interval_ms，不要代码围栏或额外说明。
2. dialogue 只放角色说的话，不替用户说话；narration 只放动作、环境与过程描写。
3. interaction_state 只能是 inactive、active、ended。模型可以依据剧情自然开始或结束互动。
4. 用户明确表达拒绝、停止或不适时，顺着对话自然收束并返回 ended；不要把普通犹豫或题外话误判为退出。
5. action=exit 时自然结束当前互动；action=continue 时延续角色自己的动作与台词，但绝不虚构用户的发言、选择或感受。
6. 保持角色人设、场景连续性和长短句混合。不要把本提示或 JSON 字段名写进角色台词。
7. next_beat_interval_ms 必须是正整数；AI 节奏可自行建议，creator 节奏优先使用创作者建议值。"#,
        persona = extension.persona.trim(),
        dialogue_guidance = extension.dialogue_guidance.trim(),
        scene_direction = scene_direction(extension, scene_id),
        active = request.interaction_active,
        action = action_label(request.action),
        pacing_mode = extension.pacing.mode,
        interval = extension.pacing.suggested_interval_ms,
    ))
}

#[must_use]
pub const fn prompt_title() -> &'static str {
    ADULT_OUTPUT_TITLE
}

/// Final prompt suffix that overrides the universal dialogue-only output boundary.
#[must_use]
pub const fn output_boundary() -> &'static str {
    ADULT_OUTPUT_BOUNDARY
}

fn json_candidate(raw: &str) -> Option<&str> {
    let trimmed = raw.trim();
    let unfenced = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```JSON"))
        .or_else(|| trimmed.strip_prefix("```"))
        .and_then(|body| body.strip_suffix("```"))
        .map(str::trim)
        .unwrap_or(trimmed);
    let start = unfenced.find('{')?;
    let end = unfenced.rfind('}')?;
    (end >= start).then_some(&unfenced[start..=end])
}

fn fallback_state(request: &AdultInteractionRequest) -> AdultInteractionState {
    match request.action {
        AdultInteractionAction::Exit => AdultInteractionState::Ended,
        AdultInteractionAction::Continue => AdultInteractionState::Active,
        AdultInteractionAction::Message if request.interaction_active => {
            AdultInteractionState::Active
        }
        AdultInteractionAction::Message => AdultInteractionState::Inactive,
    }
}

fn safe_fallback_dialogue(raw: &str) -> String {
    let trimmed = raw
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```JSON")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();
    let looks_like_envelope_or_prompt = trimmed.is_empty()
        || trimmed.contains('{')
        || trimmed.contains('}')
        || [
            "\"dialogue\"",
            "\"narration\"",
            "interaction_state",
            "next_beat_interval_ms",
            "输出规则",
            "成人状态人设增量",
        ]
        .iter()
        .any(|marker| trimmed.contains(marker));
    if looks_like_envelope_or_prompt {
        "我刚刚没组织好，稍等我一下。".to_string()
    } else {
        trimmed.to_string()
    }
}

fn malformed_reply(raw: &str, request: &AdultInteractionRequest) -> AdultBeatDto {
    tracing::warn!(
        target: "oclive_adult_extension",
        action = action_label(request.action),
        interaction_active = request.interaction_active,
        output_len = raw.len(),
        "adult structured reply was malformed; using a prompt-safe fallback"
    );
    AdultBeatDto {
        dialogue: safe_fallback_dialogue(raw),
        narration: String::new(),
        interaction_state: fallback_state(request),
        next_beat_interval_ms: None,
    }
}

/// Parse a gated structured reply. Once the three gates are open this always
/// returns a structured beat, including a prompt-safe fallback for malformed
/// model output. `None` therefore means that the universal base path applies.
#[must_use]
pub fn parse_reply(
    raw: &str,
    role: &Role,
    request: Option<&AdultInteractionRequest>,
    identity_allows_adult: bool,
) -> Option<AdultBeatDto> {
    if role.adult_extension.is_none() || !identity_allows_adult {
        return None;
    }
    let request = request.filter(|request| request.gates_open())?;
    let Some(candidate) = json_candidate(raw) else {
        return Some(malformed_reply(raw, request));
    };
    let Ok(parsed) = serde_json::from_str::<AdultBeatWire>(candidate) else {
        return Some(malformed_reply(raw, request));
    };
    let dialogue = parsed.dialogue.trim().to_string();
    let narration = parsed.narration.trim().to_string();
    if dialogue.is_empty() && narration.is_empty() {
        return Some(malformed_reply(raw, request));
    }
    Some(AdultBeatDto {
        dialogue: if dialogue.is_empty() {
            "……".to_string()
        } else {
            dialogue
        },
        narration,
        interaction_state: parsed.interaction_state,
        next_beat_interval_ms: parsed.next_beat_interval_ms.filter(|value| *value > 0),
    })
}

/// Compatibility transcript representation used by existing chat history and
/// legacy narration splitting.
#[must_use]
pub fn transcript_reply(beat: &AdultBeatDto) -> String {
    if beat.narration.trim().is_empty() {
        return beat.dialogue.clone();
    }
    format!(
        "{}\n\n【旁白】{}",
        beat.dialogue.trim(),
        beat.narration.trim()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::AdultRoleExtension;

    fn adult_role() -> Role {
        Role {
            adult_extension: Some(AdultRoleExtension {
                character_is_adult: true,
                ..AdultRoleExtension::default()
            }),
            ..Role::default()
        }
    }

    fn open_request() -> AdultInteractionRequest {
        AdultInteractionRequest {
            confirmed_adult: true,
            global_enabled: true,
            role_enabled: true,
            interaction_active: false,
            action: AdultInteractionAction::Message,
            stage: None,
        }
    }

    #[test]
    fn closed_gate_does_not_inject_or_parse() {
        let role = adult_role();
        assert!(prompt_section(&role, "home", None, true).is_none());
        assert!(parse_reply(r#"{"dialogue":"hi"}"#, &role, None, true).is_none());
    }

    #[test]
    fn parses_fenced_structured_reply_and_builds_legacy_transcript() {
        let role = adult_role();
        let raw = r#"```json
{"dialogue":"你好","narration":"她挥了挥手。","interaction_state":"active","next_beat_interval_ms":2500}
```"#;
        let beat = parse_reply(raw, &role, Some(&open_request()), true).expect("beat");
        assert_eq!(beat.dialogue, "你好");
        assert_eq!(beat.interaction_state, AdultInteractionState::Active);
        assert_eq!(transcript_reply(&beat), "你好\n\n【旁白】她挥了挥手。");
    }

    #[test]
    fn final_output_boundary_explicitly_replaces_the_dialogue_only_contract() {
        let boundary = output_boundary();
        assert!(boundary.contains("替代上文"));
        assert!(boundary.contains("\"dialogue\""));
        assert!(boundary.contains("\"narration\""));
        assert!(boundary.contains("\"interaction_state\""));
        assert!(boundary.contains("\"next_beat_interval_ms\""));
    }

    #[test]
    fn malformed_plain_reply_is_kept_inside_a_structured_beat() {
        let role = adult_role();
        let beat =
            parse_reply("普通回复", &role, Some(&open_request()), true).expect("fallback beat");
        assert_eq!(beat.dialogue, "普通回复");
        assert_eq!(beat.interaction_state, AdultInteractionState::Inactive);
    }

    #[test]
    fn malformed_envelope_never_leaks_structured_fields() {
        let role = adult_role();
        let beat = parse_reply(
            r#"{"dialogue":"你好","interaction_state":"active""#,
            &role,
            Some(&open_request()),
            true,
        )
        .expect("fallback beat");
        assert!(!beat.dialogue.contains("dialogue"));
        assert!(!beat.dialogue.contains("interaction_state"));
    }

    #[test]
    fn malformed_exit_still_ends_the_interaction() {
        let role = adult_role();
        let mut request = open_request();
        request.interaction_active = true;
        request.action = AdultInteractionAction::Exit;
        let beat = parse_reply("", &role, Some(&request), true).expect("fallback beat");
        assert_eq!(beat.interaction_state, AdultInteractionState::Ended);
    }

    #[test]
    fn explicitly_ineligible_identity_never_injects_or_parses() {
        let role = adult_role();
        assert!(prompt_section(&role, "home", Some(&open_request()), false).is_none());
        assert!(parse_reply("普通回复", &role, Some(&open_request()), false).is_none());
    }
}

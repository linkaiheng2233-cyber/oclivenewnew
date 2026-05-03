//! 虚拟时间：`get_time_state` / `jump_time`、独白生成与 OOCP 最小结果封装。

use crate::domain::user_identity::resolve_effective_user_relation_key;
use crate::error::{AppError, Result};
use crate::models::dto::{
    GenerateMonologueResponse, JumpTimeRequest, JumpTimeResponse, TimeStateResponse,
};
use crate::models::Role;
use crate::state::KernelAppState;
use chrono::{DateTime, Local, Timelike, Utc};
use serde_json::{json, Value};

/// 虚拟时间对齐到分钟（毫秒时间戳）
pub fn round_to_minute_ms(ts_ms: i64) -> i64 {
    const M: i64 = 60_000;
    (ts_ms / M) * M
}

fn virtual_time_label_from_ms(ms: i64) -> String {
    let dt = DateTime::from_timestamp_millis(ms).unwrap_or_else(Utc::now);
    dt.with_timezone(&Local).format("%Y-%m-%d %H:%M").to_string()
}

/// 角色包 `settings.json` → `autonomous_scene`：虚拟时间变化后尝试匹配首条规则并更新 `current_scene`。
async fn apply_autonomous_scene_after_jump(
    state: &KernelAppState,
    role_id: &str,
    role: &Role,
    virtual_time_ms: i64,
) -> Result<Option<(String, String)>> {
    let Some(ref cfg) = role.autonomous_scene else {
        return Ok(None);
    };
    if cfg.on_virtual_time.is_empty() {
        return Ok(None);
    }
    let current = state.db_manager.get_current_scene(role_id).await?;
    let Some(cs) = current else {
        return Ok(None);
    };
    let hour = DateTime::from_timestamp_millis(virtual_time_ms)
        .map(|d| d.hour() as u8)
        .unwrap_or(0);

    for rule in &cfg.on_virtual_time {
        if rule.when_scene != cs {
            continue;
        }
        let in_win = if rule.hour_start < rule.hour_end {
            hour >= rule.hour_start && hour < rule.hour_end
        } else {
            hour >= rule.hour_start || hour < rule.hour_end
        };
        if !in_win {
            continue;
        }
        let scenes = state.storage.list_scene_ids(role_id)?;
        if !scenes.iter().any(|s| s == &rule.to_scene) {
            continue;
        }
        if !state
            .storage
            .is_scene_time_allowed(role_id, rule.to_scene.as_str(), virtual_time_ms)
        {
            continue;
        }
        state
            .db_manager
            .set_current_scene(role_id, &rule.to_scene)
            .await?;
        return Ok(Some((cs, rule.to_scene.clone())));
    }
    Ok(None)
}

fn resolve_preset_target_ms(base_ms: i64, preset_raw: &str) -> Option<i64> {
    let mut dt = DateTime::from_timestamp_millis(base_ms)?;
    let preset = preset_raw.trim().to_ascii_lowercase();
    match preset.as_str() {
        "+2h" => Some(base_ms + 2 * 60 * 60 * 1000),
        "+6h" | "skip_idle_time" => Some(base_ms + 6 * 60 * 60 * 1000),
        "next_morning" => {
            dt += chrono::Duration::days(1);
            dt = dt
                .with_hour(8)?
                .with_minute(0)?
                .with_second(0)?
                .with_nanosecond(0)?;
            Some(dt.timestamp_millis())
        }
        _ => None,
    }
}

pub async fn get_time_state(state: &KernelAppState, role_id: &str) -> Result<TimeStateResponse> {
    if !state.db_manager.role_runtime_exists(role_id).await? {
        return Err(AppError::InvalidParameter(
            "Role runtime not initialized; call load_role first".to_string(),
        ));
    }

    if !state.db_manager.get_interaction_mode(role_id).await?.is_immersive() {
        let ms = round_to_minute_ms(Utc::now().timestamp_millis());
        let dt = DateTime::from_timestamp_millis(ms).unwrap_or_else(Utc::now);
        return Ok(TimeStateResponse {
            virtual_time_ms: ms,
            iso_datetime: dt.to_rfc3339(),
        });
    }

    let mut ms = state
        .db_manager
        .get_virtual_time_ms(role_id)
        .await?
        .unwrap_or(0);
    if ms == 0 {
        ms = round_to_minute_ms(Utc::now().timestamp_millis());
        state.db_manager.set_virtual_time_ms(role_id, ms).await?;
    }

    let dt = DateTime::from_timestamp_millis(ms).unwrap_or_else(Utc::now);
    Ok(TimeStateResponse {
        virtual_time_ms: ms,
        iso_datetime: dt.to_rfc3339(),
    })
}

/// 时间跳转后批量生成独白（使用已算好的虚拟时间，避免循环依赖 `time` ↔ `monologue`）
pub async fn generate_monologue_lines(
    state: &KernelAppState,
    role_id: &str,
    ts: &TimeStateResponse,
    count: usize,
) -> Result<Vec<String>> {
    if count == 0 {
        return Ok(vec![]);
    }
    let role = state.load_role_cached(role_id)?;
    let scene = state
        .db_manager
        .get_current_scene(role_id)
        .await?
        .unwrap_or_else(|| "default".to_string());

    let templates = state.storage.scene_monologue_templates(role_id, &scene);
    let hint = if templates.is_empty() {
        String::new()
    } else {
        format!(
            "\n可参考场景独白模板（可化用语气，不必照抄）：\n{}\n",
            templates.join("\n")
        )
    };

    let prompts: Vec<String> = (0..count.min(3))
        .map(|i| {
            if i == 0 {
                format!(
                    "你是「{}」。当前虚拟时间：{}。当前场景 id：{}{}\
                    \n请用第一人称写一句简短的内心独白（中文，35 字以内），不要加引号或前缀。",
                    role.name, ts.iso_datetime, scene, hint
                )
            } else if i == 1 {
                format!(
                    "你是「{}」。情绪延续上一刻，用另一种口吻再写一句内心独白（中文，35 字以内），不要加引号。",
                    role.name
                )
            } else {
                format!(
                    "你是「{}」。从更细微的感受再写一句独白（中文，30 字以内），不要加引号。",
                    role.name
                )
            }
        })
        .collect();

    let session_ns = format!("{}__sess__default", role_id);
    let pl = state.resolved_plugins_for_session(role.as_ref(), Some(session_ns.as_str()));
    let ollama_model = role.resolve_ollama_model(state.global_chat_model().as_str());
    let mut out = Vec::new();
    for (i, p) in prompts.into_iter().enumerate() {
        let text = match pl.llm.generate(ollama_model.as_str(), &p).await {
            Ok(s) => s,
            Err(e) => {
                if !templates.is_empty() {
                    let idx = (ts.virtual_time_ms as usize)
                        .wrapping_add(i)
                        .wrapping_add(templates.len())
                        % templates.len();
                    log::warn!("jump monologue LLM failed, scene template [{}]: {}", idx, e);
                    templates[idx].clone()
                } else {
                    return Err(AppError::OllamaError(e.to_string()));
                }
            }
        };
        let t = text.trim().to_string();
        if !t.is_empty() {
            out.push(t);
        }
    }
    Ok(out)
}

pub async fn jump_time(state: &KernelAppState, req: &JumpTimeRequest) -> Result<JumpTimeResponse> {
    if !state.db_manager.role_runtime_exists(&req.role_id).await? {
        return Err(AppError::InvalidParameter(
            "Role runtime not initialized; call load_role first".to_string(),
        ));
    }

    let role = state.load_role_cached(&req.role_id)?;
    let current_scene = state
        .db_manager
        .get_current_scene(&req.role_id)
        .await?;
    let eff_key = resolve_effective_user_relation_key(
        state,
        role.as_ref(),
        &req.role_id,
        current_scene.as_deref(),
    )
    .await?;

    let favor_before = state
        .db_manager
        .favorability_for_identity_with_runtime_fallback(&req.role_id, eff_key.as_str())
        .await?;

    if !state
        .db_manager
        .get_interaction_mode(&req.role_id)
        .await?
        .is_immersive()
    {
        let ms = round_to_minute_ms(Utc::now().timestamp_millis());
        let dt = DateTime::from_timestamp_millis(ms).unwrap_or_else(Utc::now);
        return Ok(JumpTimeResponse {
            virtual_time_ms: ms,
            iso_datetime: dt.to_rfc3339(),
            monologues: vec![],
            favorability_delta: 0.0,
            favorability_current: favor_before as f32,
            autonomous_scene_from: None,
            autonomous_scene_to: None,
        });
    }

    let base_ms = state
        .db_manager
        .get_virtual_time_ms(&req.role_id)
        .await?
        .unwrap_or_else(|| round_to_minute_ms(Utc::now().timestamp_millis()));
    let target_ms = match (req.timestamp_ms, req.preset.as_deref()) {
        (Some(ts), _) => ts,
        (None, Some(preset)) => resolve_preset_target_ms(base_ms, preset).ok_or_else(|| {
            AppError::InvalidParameter(format!("unsupported jump preset: {}", preset))
        })?,
        (None, None) => {
            return Err(AppError::InvalidParameter(
                "jump_time requires timestamp_ms or preset".to_string(),
            ));
        }
    };
    let ms = round_to_minute_ms(target_ms);
    state
        .db_manager
        .set_virtual_time_ms(&req.role_id, ms)
        .await?;
    let autonomous_scene =
        apply_autonomous_scene_after_jump(state, &req.role_id, role.as_ref(), ms).await?;
    let ts = get_time_state(state, &req.role_id).await?;

    let favor_after = state
        .db_manager
        .favorability_for_identity_with_runtime_fallback(&req.role_id, eff_key.as_str())
        .await?;

    let monologues = generate_monologue_lines(state, &req.role_id, &ts, 2).await?;

    let delta = (favor_after - favor_before) as f32;

    Ok(JumpTimeResponse {
        virtual_time_ms: ts.virtual_time_ms,
        iso_datetime: ts.iso_datetime.clone(),
        monologues,
        favorability_delta: delta,
        favorability_current: favor_after as f32,
        autonomous_scene_from: autonomous_scene.as_ref().map(|(a, _)| a.clone()),
        autonomous_scene_to: autonomous_scene.as_ref().map(|(_, b)| b.clone()),
    })
}

/// 单条独白（Tauri / OOCP `chat.generate_monologue`）
pub async fn generate_monologue(
    state: &KernelAppState,
    role_id: &str,
    context: Option<&str>,
) -> Result<GenerateMonologueResponse> {
    if !state.db_manager.role_runtime_exists(role_id).await? {
        return Err(AppError::InvalidParameter(
            "Role runtime not initialized; call load_role first".to_string(),
        ));
    }

    let role = state.load_role_cached(role_id)?;
    let scene = state
        .db_manager
        .get_current_scene(role_id)
        .await?
        .unwrap_or_else(|| "default".to_string());
    let ts = get_time_state(state, role_id).await?;

    let templates = state.storage.scene_monologue_templates(role_id, &scene);
    let hint = if templates.is_empty() {
        String::new()
    } else {
        format!(
            "\n可参考场景独白模板（可化用语气，不必照抄）：\n{}\n",
            templates.join("\n")
        )
    };

    let ctx_line = context
        .filter(|s| !s.trim().is_empty())
        .map(|c| format!("\n触发/背景：{}\n", c))
        .unwrap_or_default();

    let prompt = format!(
        "你是「{}」。当前虚拟时间：{}。当前场景 id：{}{}{}\
        \n请用第一人称写一句简短的内心独白（中文，35 字以内），不要加引号或前缀。",
        role.name, ts.iso_datetime, scene, hint, ctx_line
    );

    let session_ns = format!("{}__sess__default", role_id);
    let pl = state.resolved_plugins_for_session(role.as_ref(), Some(session_ns.as_str()));
    let ollama_model = role.resolve_ollama_model(state.global_chat_model().as_str());
    let text = match pl.llm.generate(ollama_model.as_str(), &prompt).await {
        Ok(s) => s,
        Err(e) => {
            if !templates.is_empty() {
                let idx =
                    (ts.virtual_time_ms as usize).wrapping_add(templates.len()) % templates.len();
                log::warn!(
                    "monologue LLM failed, using scene template [{}]: {}",
                    idx,
                    e
                );
                templates[idx].clone()
            } else {
                return Err(AppError::OllamaError(e.to_string()));
            }
        }
    };

    Ok(GenerateMonologueResponse {
        text: text.trim().to_string(),
    })
}

/// OOCP `time.get_state` 结果（扩展字段与 v0.1 文档对齐）
pub async fn get_time_state_oocp_value(state: &KernelAppState, role_id: &str) -> Result<Value> {
    let ts = get_time_state(state, role_id).await?;
    let mode = state.db_manager.get_interaction_mode(role_id).await?;
    let current_scene = state
        .db_manager
        .get_current_scene(role_id)
        .await?
        .unwrap_or_else(|| "default".to_string());
    let label = virtual_time_label_from_ms(ts.virtual_time_ms);
    Ok(json!({
        "virtual_time_ms": ts.virtual_time_ms,
        "virtual_time_label": label,
        "iso_datetime": ts.iso_datetime,
        "speed_multiplier": 1.0,
        "paused": false,
        "character_current_scene": current_scene,
        "character_interaction_mode": mode.as_str(),
    }))
}

/// OOCP `time.jump`：`target_time_ms` 与 `preset` 至少其一（两者皆有时以 `target_time_ms` 为准）
pub async fn jump_time_oocp_from_params(
    state: &KernelAppState,
    role_id: &str,
    target_time_ms: Option<i64>,
    preset: Option<&str>,
) -> Result<Value> {
    let req = JumpTimeRequest {
        role_id: role_id.to_string(),
        timestamp_ms: target_time_ms,
        preset: preset.map(|s| s.to_string()),
    };
    let resp = jump_time(state, &req).await?;
    let label = virtual_time_label_from_ms(resp.virtual_time_ms);
    Ok(json!({
        "virtual_time_ms": resp.virtual_time_ms,
        "virtual_time_label": label,
        "iso_datetime": resp.iso_datetime,
        "monologues": resp.monologues,
        "favorability_delta": resp.favorability_delta,
        "favorability_current": resp.favorability_current,
        "autonomous_scene_from": resp.autonomous_scene_from,
        "autonomous_scene_to": resp.autonomous_scene_to,
    }))
}

#[cfg(test)]
mod tests {
    use super::{resolve_preset_target_ms, round_to_minute_ms};

    #[test]
    fn round_to_minute_ms_aligns_down() {
        assert_eq!(round_to_minute_ms(60_000), 60_000);
        assert_eq!(round_to_minute_ms(60_001), 60_000);
        assert_eq!(round_to_minute_ms(119_999), 60_000);
        assert_eq!(round_to_minute_ms(0), 0);
        assert_eq!(round_to_minute_ms(-60_001), -60_000);
    }

    #[test]
    fn resolve_preset_target_ms_supports_offsets() {
        let base = 1_700_000_000_000_i64;
        assert_eq!(
            resolve_preset_target_ms(base, "+2h"),
            Some(base + 2 * 60 * 60 * 1000)
        );
        assert_eq!(
            resolve_preset_target_ms(base, "skip_idle_time"),
            Some(base + 6 * 60 * 60 * 1000)
        );
    }
}

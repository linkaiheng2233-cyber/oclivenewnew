//! Virtual time and monologue generation shared by HTTP routes and Tauri invoke.

use crate::command_error::CommandError;
use crate::domain::user_identity::resolve_effective_user_relation_key;
use crate::domain::virtual_time_sync::{apply_virtual_time_jump, sync_and_persist_virtual_time};
use crate::error::AppError;
use crate::models::dto::{
    GenerateMonologueResponse, JumpTimeRequest, JumpTimeResponse, TimeStateResponse,
};
use crate::models::Role;
use crate::service::role::ensure_manifest_role_ready;
use crate::state::AppState;
use chrono::{DateTime, Timelike, Utc};
use oclive_kernel_runtime::domain::virtual_time::round_to_minute_ms;

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
pub async fn get_time_state_impl(
    state: &AppState,
    role_id: &str,
) -> Result<TimeStateResponse, CommandError> {
    ensure_manifest_role_ready(state, role_id).await?;

    let role = state.load_role_cached_async(role_id).await?;
    let immersive = state
        .db_manager
        .get_interaction_mode(role_id)
        .await?
        .is_immersive();
    let ms =
        sync_and_persist_virtual_time(state.db_manager.as_ref(), role.as_ref(), role_id, immersive)
            .await?;
    let dt = DateTime::from_timestamp_millis(ms).unwrap_or_else(Utc::now);
    Ok(TimeStateResponse {
        virtual_time_ms: ms,
        iso_datetime: dt.to_rfc3339(),
    })
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

async fn apply_autonomous_scene_after_jump(
    state: &AppState,
    role_id: &str,
    role: &Role,
    virtual_time_ms: i64,
) -> Result<Option<(String, String)>, CommandError> {
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
        if !state.storage.is_scene_time_allowed_for_role(
            role,
            rule.to_scene.as_str(),
            virtual_time_ms,
        ) {
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

/// Batch-generate monologue lines after time jump.
pub async fn generate_monologue_lines(
    state: &AppState,
    role_id: &str,
    ts: &TimeStateResponse,
    count: usize,
) -> Result<Vec<String>, CommandError> {
    if count == 0 {
        return Ok(vec![]);
    }
    let role = state.load_role_cached_async(role_id).await?;
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

    let pl = state.resolved_plugins_for_session(role.as_ref(), Some(role_id));
    let ollama_model = role.resolve_ollama_model(state.ollama_model.as_str());
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
                    tracing::warn!("jump monologue LLM failed, scene template [{}]: {}", idx, e);
                    templates[idx].clone()
                } else {
                    return Err(AppError::OllamaError(e.to_string()).into());
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

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
pub async fn generate_monologue_impl(
    state: &AppState,
    role_id: &str,
) -> Result<GenerateMonologueResponse, CommandError> {
    ensure_manifest_role_ready(state, role_id).await?;
    let ts = get_time_state_impl(state, role_id).await?;
    let lines = generate_monologue_lines(state, role_id, &ts, 1).await?;
    Ok(GenerateMonologueResponse {
        text: lines.into_iter().next().unwrap_or_default(),
    })
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
pub async fn jump_time_impl(
    state: &AppState,
    req: &JumpTimeRequest,
) -> Result<JumpTimeResponse, CommandError> {
    ensure_manifest_role_ready(state, &req.role_id).await?;

    let role = state.load_role_cached_async(&req.role_id).await?;
    let current_scene = state.db_manager.get_current_scene(&req.role_id).await?;
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

    let immersive = state
        .db_manager
        .get_interaction_mode(&req.role_id)
        .await?
        .is_immersive();

    if !immersive {
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

    let base_ms =
        sync_and_persist_virtual_time(state.db_manager.as_ref(), role.as_ref(), &req.role_id, true)
            .await?;
    let target_ms = match (req.timestamp_ms, req.preset.as_deref()) {
        (Some(ts), _) => ts,
        (None, Some(preset)) => resolve_preset_target_ms(base_ms, preset).ok_or_else(|| {
            AppError::InvalidParameter(format!("unsupported jump preset: {preset}"))
        })?,
        (None, None) => {
            return Err(AppError::InvalidParameter(
                "jump_time requires timestamp_ms or preset".to_string(),
            )
            .into());
        }
    };
    let ms =
        apply_virtual_time_jump(state, role.as_ref(), &req.role_id, base_ms, target_ms).await?;
    let autonomous_scene =
        apply_autonomous_scene_after_jump(state, &req.role_id, role.as_ref(), ms).await?;
    let ts = get_time_state_impl(state, &req.role_id).await?;

    let favor_after = state
        .db_manager
        .favorability_for_identity_with_runtime_fallback(&req.role_id, eff_key.as_str())
        .await?;

    let monologues = generate_monologue_lines(state, &req.role_id, &ts, 2).await?;
    let delta = (favor_after - favor_before) as f32;

    Ok(JumpTimeResponse {
        virtual_time_ms: ts.virtual_time_ms,
        iso_datetime: ts.iso_datetime,
        monologues,
        favorability_delta: delta,
        favorability_current: favor_after as f32,
        autonomous_scene_from: autonomous_scene.as_ref().map(|(a, _)| a.clone()),
        autonomous_scene_to: autonomous_scene.as_ref().map(|(_, b)| b.clone()),
    })
}

#[cfg(test)]
mod tests {
    use super::resolve_preset_target_ms;
    use oclive_kernel_runtime::domain::virtual_time::round_to_minute_ms;

    #[test]
    fn round_to_minute_ms_aligns_down() {
        assert_eq!(round_to_minute_ms(60_000), 60_000);
        assert_eq!(round_to_minute_ms(60_001), 60_000);
    }

    #[test]
    fn resolve_preset_target_ms_supports_offsets() {
        let base = 1_700_000_000_000_i64;
        assert_eq!(
            resolve_preset_target_ms(base, "+2h"),
            Some(base + 2 * 60 * 60 * 1000)
        );
    }
}

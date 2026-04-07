use crate::api::jump_monologue::generate_monologue_lines;
use crate::domain::user_identity::resolve_effective_user_relation_key;
use crate::error::AppError;
use crate::models::dto::{JumpTimeRequest, JumpTimeResponse, TimeStateResponse};
use crate::models::Role;
use crate::state::AppState;
use chrono::{DateTime, Timelike, Utc};
use tauri::State;

/// 虚拟时间对齐到分钟（毫秒时间戳）
pub fn round_to_minute_ms(ts_ms: i64) -> i64 {
    const M: i64 = 60_000;
    (ts_ms / M) * M
}

/// 角色包 `settings.json` → `autonomous_scene`：虚拟时间变化后尝试匹配首条规则并更新 `current_scene`。
/// 若发生切换，返回 `(from_scene_id, to_scene_id)`。
async fn apply_autonomous_scene_after_jump(
    state: &AppState,
    role_id: &str,
    role: &Role,
    virtual_time_ms: i64,
) -> Result<Option<(String, String)>, String> {
    let Some(ref cfg) = role.autonomous_scene else {
        return Ok(None);
    };
    if cfg.on_virtual_time.is_empty() {
        return Ok(None);
    }
    let current = state
        .db_manager
        .get_current_scene(role_id)
        .await
        .map_err(|e| e.to_frontend_error())?;
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
        let scenes = state
            .storage
            .list_scene_ids(role_id)
            .map_err(|e| e.to_frontend_error())?;
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
            .await
            .map_err(|e| e.to_frontend_error())?;
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

pub async fn get_time_state_impl(
    state: &AppState,
    role_id: &str,
) -> Result<TimeStateResponse, String> {
    if !state
        .db_manager
        .role_runtime_exists(role_id)
        .await
        .map_err(|e| e.to_frontend_error())?
    {
        return Err(AppError::InvalidParameter(
            "Role runtime not initialized; call load_role first".to_string(),
        )
        .to_frontend_error());
    }

    if !state
        .db_manager
        .get_interaction_mode(role_id)
        .await
        .map_err(|e| e.to_frontend_error())?
        .is_immersive()
    {
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
        .await
        .map_err(|e| e.to_frontend_error())?
        .unwrap_or(0);
    if ms == 0 {
        ms = round_to_minute_ms(Utc::now().timestamp_millis());
        state
            .db_manager
            .set_virtual_time_ms(role_id, ms)
            .await
            .map_err(|e| e.to_frontend_error())?;
    }

    let dt = DateTime::from_timestamp_millis(ms).unwrap_or_else(Utc::now);
    Ok(TimeStateResponse {
        virtual_time_ms: ms,
        iso_datetime: dt.to_rfc3339(),
    })
}

pub async fn jump_time_impl(
    state: &AppState,
    req: &JumpTimeRequest,
) -> Result<JumpTimeResponse, String> {
    if !state
        .db_manager
        .role_runtime_exists(&req.role_id)
        .await
        .map_err(|e| e.to_frontend_error())?
    {
        return Err(AppError::InvalidParameter(
            "Role runtime not initialized; call load_role first".to_string(),
        )
        .to_frontend_error());
    }

    let role = state
        .storage
        .load_role(&req.role_id)
        .map_err(|e| e.to_frontend_error())?;
    let current_scene = state
        .db_manager
        .get_current_scene(&req.role_id)
        .await
        .map_err(|e| e.to_frontend_error())?;
    let eff_key =
        resolve_effective_user_relation_key(state, &role, &req.role_id, current_scene.as_deref())
            .await
            .map_err(|e| e.to_frontend_error())?;

    let favor_before = state
        .db_manager
        .favorability_for_identity_with_runtime_fallback(&req.role_id, eff_key.as_str())
        .await
        .map_err(|e| e.to_frontend_error())?;

    if !state
        .db_manager
        .get_interaction_mode(&req.role_id)
        .await
        .map_err(|e| e.to_frontend_error())?
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
        .await
        .map_err(|e| e.to_frontend_error())?
        .unwrap_or_else(|| round_to_minute_ms(Utc::now().timestamp_millis()));
    let target_ms = match (req.timestamp_ms, req.preset.as_deref()) {
        (Some(ts), _) => ts,
        (None, Some(preset)) => resolve_preset_target_ms(base_ms, preset).ok_or_else(|| {
            AppError::InvalidParameter(format!("unsupported jump preset: {}", preset))
                .to_frontend_error()
        })?,
        (None, None) => {
            return Err(AppError::InvalidParameter(
                "jump_time requires timestamp_ms or preset".to_string(),
            )
            .to_frontend_error());
        }
    };
    let ms = round_to_minute_ms(target_ms);
    state
        .db_manager
        .set_virtual_time_ms(&req.role_id, ms)
        .await
        .map_err(|e| e.to_frontend_error())?;
    let autonomous_scene =
        apply_autonomous_scene_after_jump(state, &req.role_id, &role, ms).await?;
    let ts = get_time_state_impl(state, &req.role_id).await?;

    let favor_after = state
        .db_manager
        .favorability_for_identity_with_runtime_fallback(&req.role_id, eff_key.as_str())
        .await
        .map_err(|e| e.to_frontend_error())?;

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

#[tauri::command]
pub async fn get_time_state(
    role_id: String,
    state: State<'_, AppState>,
) -> Result<TimeStateResponse, String> {
    get_time_state_impl(&state, &role_id).await
}

#[tauri::command]
pub async fn jump_time(
    req: JumpTimeRequest,
    state: State<'_, AppState>,
) -> Result<JumpTimeResponse, String> {
    jump_time_impl(&state, &req).await
}

#[cfg(test)]
mod tests {
    use super::resolve_preset_target_ms;
    use super::round_to_minute_ms;

    #[test]
    fn round_to_minute_ms_aligns_down() {
        assert_eq!(round_to_minute_ms(60_000), 60_000);
        assert_eq!(round_to_minute_ms(60_001), 60_000);
        assert_eq!(round_to_minute_ms(119_999), 60_000);
        assert_eq!(round_to_minute_ms(0), 0);
        // 负时间戳：整除向 0 截断，与 Rust `/` 一致
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

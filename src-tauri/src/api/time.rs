use crate::api::jump_monologue::generate_monologue_lines;
use crate::api::role::ensure_manifest_role_ready;
use crate::domain::user_identity::resolve_effective_user_relation_key;
use crate::domain::virtual_time_sync::{apply_virtual_time_jump, sync_and_persist_virtual_time};
use crate::error::AppError;
use crate::kernel_attach::KernelHttpClient;
use crate::kernel_lifecycle::SharedKernelConnection;
use crate::models::dto::{JumpTimeRequest, JumpTimeResponse, TimeStateResponse};
use crate::models::Role;
use crate::state::{AppState, SharedAppState};
use chrono::{DateTime, Timelike, Utc};
use oclive_kernel_runtime::domain::virtual_time::round_to_minute_ms;
use tauri::{AppHandle, Manager, State};
use crate::api::error::CommandError;

async fn get_time_state_via_kernel(
    conn: &SharedKernelConnection,
    role_id: &str,
) -> Result<TimeStateResponse, AppError> {
    match KernelHttpClient::get_time_state_via_http(conn, role_id).await {
        Ok(ts) => Ok(ts),
        Err(e) if time_state_route_unavailable(&e) => {
            get_time_state_via_role_info(conn, role_id).await
        }
        Err(AppError::RoleRuntimeNotReady) => {
            KernelHttpClient::load_role_via_http(conn, role_id.trim()).await?;
            match KernelHttpClient::get_time_state_via_http(conn, role_id).await {
                Ok(ts) => Ok(ts),
                Err(e) if time_state_route_unavailable(&e) => {
                    get_time_state_via_role_info(conn, role_id).await
                }
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    }
}

fn time_state_route_unavailable(err: &AppError) -> bool {
    match err {
        AppError::OllamaError(msg) => {
            msg.contains("404") || msg.contains("Not Found") || msg.contains("not found")
        }
        _ => false,
    }
}

async fn get_time_state_via_role_info(
    conn: &SharedKernelConnection,
    role_id: &str,
) -> Result<TimeStateResponse, AppError> {
    use crate::models::dto::GetRoleInfoRequest;
    let req = GetRoleInfoRequest {
        role_id: role_id.to_string(),
        session_id: None,
    };
    let info = KernelHttpClient::get_role_info_via_http(conn, &req).await?;
    let ms = info.virtual_time_ms;
    let dt = DateTime::from_timestamp_millis(ms).unwrap_or_else(Utc::now);
    Ok(TimeStateResponse {
        virtual_time_ms: ms,
        iso_datetime: dt.to_rfc3339(),
    })
}

/// Role pack `settings.json` → `autonomous_scene`: after virtual time changes, try matching the first rule and update `current_scene`.
/// Returns `(from_scene_id, to_scene_id)` when a switch occurs.
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
    let current = state
        .db_manager
        .get_current_scene(role_id)
        .await
        ?;
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
            ?;
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
            .await
            ?;
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
        .await
        ?
        .is_immersive();
    let ms = sync_and_persist_virtual_time(
        state.db_manager.as_ref(),
        role.as_ref(),
        role_id,
        immersive,
    )
        .await?;
    let dt = DateTime::from_timestamp_millis(ms).unwrap_or_else(Utc::now);
    Ok(TimeStateResponse {
        virtual_time_ms: ms,
        iso_datetime: dt.to_rfc3339(),
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

    let role = state
        .load_role_cached_async(&req.role_id)
        .await
        ?;
    let current_scene = state
        .db_manager
        .get_current_scene(&req.role_id)
        .await
        ?;
    let eff_key = resolve_effective_user_relation_key(
        state,
        role.as_ref(),
        &req.role_id,
        current_scene.as_deref(),
    )
    .await
    ?;

    let favor_before = state
        .db_manager
        .favorability_for_identity_with_runtime_fallback(&req.role_id, eff_key.as_str())
        .await
        ?;

    let immersive = state
        .db_manager
        .get_interaction_mode(&req.role_id)
        .await
        ?
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

    let base_ms = sync_and_persist_virtual_time(
        state.db_manager.as_ref(),
        role.as_ref(),
        &req.role_id,
        true,
    )
    .await?;
    let target_ms = match (req.timestamp_ms, req.preset.as_deref()) {
        (Some(ts), _) => ts,
        (None, Some(preset)) => resolve_preset_target_ms(base_ms, preset).ok_or_else(|| {
            AppError::InvalidParameter(format!("unsupported jump preset: {}", preset))
                
        })?,
        (None, None) => {
            return Err(AppError::InvalidParameter(
                "jump_time requires timestamp_ms or preset".to_string(),
            )
            .into());
        }
    };
    let ms = apply_virtual_time_jump(state, role.as_ref(), &req.role_id, base_ms, target_ms)
        .await?;
    let autonomous_scene =
        apply_autonomous_scene_after_jump(state, &req.role_id, role.as_ref(), ms).await?;
    let ts = get_time_state_impl(state, &req.role_id).await?;

    let favor_after = state
        .db_manager
        .favorability_for_identity_with_runtime_fallback(&req.role_id, eff_key.as_str())
        .await
        ?;

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
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn get_time_state(
    role_id: String,
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<TimeStateResponse, CommandError> {
    if let Some(conn) = app.try_state::<SharedKernelConnection>() {
        return get_time_state_via_kernel(&conn, role_id.trim())
            .await
            .map_err(Into::into);
    }
    get_time_state_impl(&state, &role_id).await
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn jump_time(
    req: JumpTimeRequest,
    state: State<'_, SharedAppState>,
) -> Result<JumpTimeResponse, CommandError> {
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
        // Negative timestamps: integer division truncates toward zero, consistent with Rust `/`
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

//! 应用级设置（`app_settings`），供受控桥接更新。

use crate::error::AppError;
use crate::models::interaction_mode::InteractionMode;
use crate::state::AppState;
use serde::Serialize;
use serde_json::{json, Value};
use tauri::State;
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
pub async fn update_settings_impl(state: &AppState, params: &Value) -> Result<Value, String> {
    let obj = params
        .as_object()
        .ok_or_else(|| AppError::InvalidParameter("update_settings: params must be an object".into()).to_frontend_error())?;
    if obj.is_empty() {
        return Err(AppError::InvalidParameter("update_settings: empty object".into()).to_frontend_error());
    }
    for (k, v) in obj {
        match k.as_str() {
            "theme" | "ui_theme" => {
                let s = v
                    .as_str()
                    .ok_or_else(|| {
                        AppError::InvalidParameter(format!("update_settings: {k} must be a string"))
                            .to_frontend_error()
                    })?;
                let t = s.trim().to_ascii_lowercase();
                if !matches!(t.as_str(), "light" | "dark" | "system") {
                    return Err(
                        AppError::InvalidParameter(format!("update_settings: invalid theme {s}"))
                            .to_frontend_error(),
                    );
                }
                state
                    .db_manager
                    .upsert_app_setting("ui_theme", &t)
                    .await
                    .map_err(|e| e.to_frontend_error())?;
            }
            "interaction_mode" => {
                let s = v.as_str().ok_or_else(|| {
                    AppError::InvalidParameter(
                        "update_settings: interaction_mode must be a string".into(),
                    )
                    .to_frontend_error()
                })?;
                InteractionMode::validate_optional_pack_field(Some(s))?;
                let n = InteractionMode::normalize(Some(s));
                state
                    .db_manager
                    .upsert_app_setting("interaction_mode", n.as_str())
                    .await
                    .map_err(|e| e.to_frontend_error())?;
            }
            "remote_fallback_to_builtin" => {
                let raw = match v {
                    Value::String(s) => {
                        let t = s.trim();
                        if !matches!(t, "0" | "1") {
                            return Err(
                                AppError::InvalidParameter(format!(
                                    "update_settings: remote_fallback_to_builtin must be \"0\" or \"1\", got {s:?}"
                                ))
                                .to_frontend_error(),
                            );
                        }
                        t.to_string()
                    }
                    Value::Bool(b) => {
                        if *b {
                            "1".to_string()
                        } else {
                            "0".to_string()
                        }
                    }
                    _ => {
                        return Err(
                            AppError::InvalidParameter(
                                "update_settings: remote_fallback_to_builtin must be a string or bool"
                                    .into(),
                            )
                            .to_frontend_error(),
                        );
                    }
                };
                state
                    .db_manager
                    .upsert_app_setting("remote_fallback_to_builtin", &raw)
                    .await
                    .map_err(|e| e.to_frontend_error())?;
                let fresh = state
                    .db_manager
                    .get_app_setting("remote_fallback_to_builtin")
                    .await
                    .map_err(|e| e.to_frontend_error())?;
                state.sync_remote_fallback_from_db_value(fresh);
            }
            other => {
                return Err(
                    AppError::InvalidParameter(format!(
                        "update_settings: unsupported key {other:?} (allowed: theme, ui_theme, interaction_mode, remote_fallback_to_builtin)"
                    ))
                    .to_frontend_error(),
                );
            }
        }
    }
    Ok(json!({ "ok": true }))
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn set_remote_fallback_to_builtin(
    state: State<'_, AppState>,
    allow: bool,
) -> Result<(), String> {
    let raw = if allow { "1" } else { "0" };
    state
        .db_manager
        .upsert_app_setting("remote_fallback_to_builtin", raw)
        .await
        .map_err(|e| e.to_frontend_error())?;
    let fresh = state
        .db_manager
        .get_app_setting("remote_fallback_to_builtin")
        .await
        .map_err(|e| e.to_frontend_error())?;
    state.sync_remote_fallback_from_db_value(fresh);
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteFallbackAppSettings {
    /// 数据库中的 `app_settings.remote_fallback_to_builtin`（`"0"` / `"1"`）。
    pub remote_fallback_to_builtin: String,
    /// 若设置了 `OCLIVE_REMOTE_FALLBACK_TO_BUILTIN`，进程内有效值由环境变量决定，UI 应锁定开关。
    pub remote_fallback_env_override_active: bool,
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn get_remote_fallback_app_settings(
    state: State<'_, AppState>,
) -> Result<RemoteFallbackAppSettings, String> {
    let remote_fallback_to_builtin = state
        .db_manager
        .get_app_setting("remote_fallback_to_builtin")
        .await
        .map_err(|e| e.to_frontend_error())?
        .unwrap_or_else(|| "1".to_string());
    let remote_fallback_env_override_active =
        crate::infrastructure::remote_fallback_policy::remote_fallback_env_override().is_some();
    Ok(RemoteFallbackAppSettings {
        remote_fallback_to_builtin,
        remote_fallback_env_override_active,
    })
}

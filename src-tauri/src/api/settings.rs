//! App-level settings (`app_settings`), updatable via controlled bridge.

use crate::error::AppError;
use crate::models::interaction_mode::InteractionMode;
use crate::state::AppState;
use serde::Serialize;
use serde_json::{json, Value};
use tauri::State;
use crate::api::error::CommandError;
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
pub async fn update_settings_impl(state: &AppState, params: &Value) -> Result<Value, CommandError> {
    let obj = params.as_object().ok_or_else(|| {
        CommandError::from(AppError::InvalidParameter(
            "update_settings: params must be an object".into(),
        ))
    })?;
    if obj.is_empty() {
        return Err(AppError::InvalidParameter("update_settings: empty object".into()).into());
    }
    for (k, v) in obj {
        match k.as_str() {
            "theme" | "ui_theme" => {
                let s = v.as_str().ok_or_else(|| {
                    CommandError::from(AppError::InvalidParameter(format!(
                        "update_settings: {k} must be a string"
                    )))
                })?;
                let t = s.trim().to_ascii_lowercase();
                if !matches!(t.as_str(), "light" | "dark" | "system") {
                    return Err(AppError::InvalidParameter(format!(
                        "update_settings: invalid theme {s}"
                    ))
                    .into());
                }
                state
                    .db_manager
                    .upsert_app_setting("ui_theme", &t)
                    .await
                    ?;
            }
            "interaction_mode" => {
                let s = v.as_str().ok_or_else(|| {
                    CommandError::from(AppError::InvalidParameter(
                        "update_settings: interaction_mode must be a string".into(),
                    ))
                })?;
                InteractionMode::validate_optional_pack_field(Some(s))?;
                let n = InteractionMode::normalize(Some(s));
                state
                    .db_manager
                    .upsert_app_setting("interaction_mode", n.as_str())
                    .await
                    ?;
            }
            "remote_fallback_to_builtin" => {
                let raw = match v {
                    Value::String(s) => {
                        let t = s.trim();
                        if !matches!(t, "0" | "1") {
                            return Err(AppError::InvalidParameter(format!(
                                "update_settings: remote_fallback_to_builtin must be \"0\" or \"1\", got {s:?}"
                            ))
                            .into());
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
                        return Err(AppError::InvalidParameter(
                            "update_settings: remote_fallback_to_builtin must be a string or bool"
                                .into(),
                        )
                        .into());
                    }
                };
                state
                    .db_manager
                    .upsert_app_setting("remote_fallback_to_builtin", &raw)
                    .await
                    ?;
                let fresh = state
                    .db_manager
                    .get_app_setting("remote_fallback_to_builtin")
                    .await
                    ?;
                state.sync_remote_fallback_from_db_value(fresh);
            }
            other => {
                return Err(AppError::InvalidParameter(format!(
                    "update_settings: unsupported key {other:?} (allowed: theme, ui_theme, interaction_mode, remote_fallback_to_builtin)"
                ))
                .into());
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
) -> Result<(), CommandError> {
    let raw = if allow { "1" } else { "0" };
    state
        .db_manager
        .upsert_app_setting("remote_fallback_to_builtin", raw)
        .await
        ?;
    let fresh = state
        .db_manager
        .get_app_setting("remote_fallback_to_builtin")
        .await
        ?;
    state.sync_remote_fallback_from_db_value(fresh);
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteFallbackAppSettings {
    /// `app_settings.remote_fallback_to_builtin` in DB (`"0"` / `"1"`).
    pub remote_fallback_to_builtin: String,
    /// When `OCLIVE_REMOTE_FALLBACK_TO_BUILTIN` is set, env drives in-process value; UI should lock the toggle.
    pub remote_fallback_env_override_active: bool,
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn get_remote_fallback_app_settings(
    state: State<'_, AppState>,
) -> Result<RemoteFallbackAppSettings, CommandError> {
    let remote_fallback_to_builtin = state
        .db_manager
        .get_app_setting("remote_fallback_to_builtin")
        .await
        ?
        .unwrap_or_else(|| "1".to_string());
    let remote_fallback_env_override_active =
        crate::infrastructure::remote_fallback_policy::remote_fallback_env_override().is_some();
    Ok(RemoteFallbackAppSettings {
        remote_fallback_to_builtin,
        remote_fallback_env_override_active,
    })
}

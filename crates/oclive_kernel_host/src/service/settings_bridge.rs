//! App-level settings (`app_settings`), updatable via controlled bridge.

use crate::command_error::CommandError;
use crate::error::AppError;
use crate::models::interaction_mode::InteractionMode;
use crate::state::AppState;
use serde_json::{json, Value};

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
                state.db_manager.upsert_app_setting("ui_theme", &t).await?;
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
                    .await?;
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
                    .await?;
                let fresh = state
                    .db_manager
                    .get_app_setting("remote_fallback_to_builtin")
                    .await?;
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

//! 应用级设置（`app_settings`），供受控桥接更新。

use crate::models::interaction_mode::InteractionMode;
use crate::state::AppState;
use serde_json::{json, Value};

pub async fn update_settings_impl(state: &AppState, params: &Value) -> Result<Value, String> {
    let obj = params
        .as_object()
        .ok_or_else(|| "update_settings: params must be an object".to_string())?;
    if obj.is_empty() {
        return Err("update_settings: empty object".to_string());
    }
    for (k, v) in obj {
        match k.as_str() {
            "theme" | "ui_theme" => {
                let s = v
                    .as_str()
                    .ok_or_else(|| format!("update_settings: {k} must be a string"))?;
                let t = s.trim().to_ascii_lowercase();
                if !matches!(t.as_str(), "light" | "dark" | "system") {
                    return Err(format!("update_settings: invalid theme {s}"));
                }
                state
                    .db_manager
                    .upsert_app_setting("ui_theme", &t)
                    .await
                    .map_err(|e| e.to_frontend_error())?;
            }
            "interaction_mode" => {
                let s = v.as_str().ok_or_else(|| {
                    "update_settings: interaction_mode must be a string".to_string()
                })?;
                InteractionMode::validate_optional_pack_field(Some(s))?;
                let n = InteractionMode::normalize(Some(s));
                state
                    .db_manager
                    .upsert_app_setting("interaction_mode", n.as_str())
                    .await
                    .map_err(|e| e.to_frontend_error())?;
            }
            other => {
                return Err(format!(
                    "update_settings: unsupported key {other:?} (allowed: theme, ui_theme, interaction_mode)"
                ));
            }
        }
    }
    Ok(json!({ "ok": true }))
}

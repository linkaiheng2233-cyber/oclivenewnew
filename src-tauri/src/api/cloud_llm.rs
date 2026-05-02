//! 应用内云端 LLM（OpenAI 兼容）与总闸 / 自动上游策略。

use crate::error::AppError;
use crate::models::dto::{CloudLlmUiSettingsPatchRequest, CloudLlmUiSettingsResponse};
use crate::state::AppState;
use oclive_kernel_runtime::infrastructure::cloud_llm::{
    load_user_cloud_llm_from_db, persist_user_cloud_llm_to_db, resolve_cloud_llm_config,
    CloudLlmConfig, CLOUD_LLM_APP_KEY_AUTO_REMOTE_LLM,
    CLOUD_LLM_APP_KEY_NETWORK_ACK, CLOUD_LLM_APP_KEY_OPENAI_BLOCKED,
};
use tauri::State;

fn to_resp(state: &AppState, cfg: Option<&CloudLlmConfig>) -> CloudLlmUiSettingsResponse {
    let (base_url, model, timeout_ms, api_key_set) = match cfg {
        Some(c) => (
            c.base_url.clone(),
            c.default_model.clone().unwrap_or_default(),
            c.timeout.as_millis().min(u128::from(u64::MAX)) as u64,
            !c.api_key.trim().is_empty(),
        ),
        None => (String::new(), String::new(), 120_000u64, false),
    };
    CloudLlmUiSettingsResponse {
        base_url,
        model,
        timeout_ms,
        api_key_set,
        openai_blocked: state.cloud_llm_runtime.openai_hard_blocked(),
        auto_remote_llm: state.cloud_llm_runtime.auto_remote_llm_enabled(),
        network_acknowledged: state.cloud_llm_runtime.network_acknowledged(),
        network_granted: state.is_remote_llm_network_granted(),
    }
}

async fn cloud_llm_ui_settings_inner(state: &AppState) -> Result<CloudLlmUiSettingsResponse, String> {
    let row = load_user_cloud_llm_from_db(state.db_manager.as_ref())
        .await
        .map_err(|e| e.to_string())?;
    Ok(to_resp(state, row.as_ref()))
}

#[tauri::command]
pub async fn get_cloud_llm_ui_settings(
    state: State<'_, AppState>,
) -> Result<CloudLlmUiSettingsResponse, String> {
    cloud_llm_ui_settings_inner(&state).await
}

#[tauri::command]
pub async fn set_cloud_llm_ui_settings(
    req: CloudLlmUiSettingsPatchRequest,
    state: State<'_, AppState>,
) -> Result<CloudLlmUiSettingsResponse, String> {
    if req.clear {
        persist_user_cloud_llm_to_db(state.db_manager.as_ref(), None)
            .await
            .map_err(|e| e.to_string())?;
        state.cloud_llm_runtime.set_user_config(None);
        return cloud_llm_ui_settings_inner(&state).await;
    }

    if let Some(b) = req.openai_blocked {
        let v = if b { "1" } else { "0" };
        state
            .db_manager
            .upsert_app_setting(CLOUD_LLM_APP_KEY_OPENAI_BLOCKED, v)
            .await
            .map_err(|e| e.to_string())?;
        state.cloud_llm_runtime.set_openai_hard_blocked(b);
    }
    if let Some(b) = req.auto_remote_llm {
        let v = if b { "1" } else { "0" };
        state
            .db_manager
            .upsert_app_setting(CLOUD_LLM_APP_KEY_AUTO_REMOTE_LLM, v)
            .await
            .map_err(|e| e.to_string())?;
        state.cloud_llm_runtime.set_auto_remote_llm_enabled(b);
    }
    if let Some(b) = req.network_acknowledged {
        let v = if b { "1" } else { "0" };
        state
            .db_manager
            .upsert_app_setting(CLOUD_LLM_APP_KEY_NETWORK_ACK, v)
            .await
            .map_err(|e| e.to_string())?;
        state.cloud_llm_runtime.set_network_acknowledged(b);
    }

    let needs_credential_write = req.base_url.is_some()
        || req.api_key.as_ref().map(|s| !s.trim().is_empty()).unwrap_or(false)
        || req.model.is_some()
        || req.timeout_ms.is_some();

    if !needs_credential_write {
        return cloud_llm_ui_settings_inner(&state).await;
    }

    let existing = load_user_cloud_llm_from_db(state.db_manager.as_ref())
        .await
        .map_err(|e| e.to_string())?;

    let base_url = if let Some(ref raw) = req.base_url {
        CloudLlmConfig::validate_base_url_for_ui(raw).map_err(|e: AppError| e.to_frontend_error())?
    } else {
        existing
            .as_ref()
            .map(|c| c.base_url.clone())
            .ok_or_else(|| "cloud_llm: base_url required when saving credentials".to_string())?
    };

    let api_key = match req.api_key.as_deref() {
        Some(k) if !k.trim().is_empty() => k.trim().to_string(),
        _ => existing
            .as_ref()
            .map(|c| c.api_key.clone())
            .filter(|k| !k.trim().is_empty())
            .ok_or_else(|| {
                "cloud_llm: api_key required on first setup (or leave blank only when updating key)"
                    .to_string()
            })?,
    };

    let default_model = match &req.model {
        None => existing.as_ref().and_then(|c| c.default_model.clone()),
        Some(s) => {
            let t = s.trim();
            if t.is_empty() {
                None
            } else {
                Some(t.to_string())
            }
        }
    };

    let timeout_ms = match req.timeout_ms {
        Some(ms) => ms.clamp(1_000, 600_000),
        None => existing
            .as_ref()
            .map(|c| c.timeout.as_millis().min(u128::from(u64::MAX)) as u64)
            .unwrap_or(120_000),
    };

    let cfg = CloudLlmConfig {
        base_url,
        api_key,
        timeout: std::time::Duration::from_millis(timeout_ms),
        default_model,
    };
    persist_user_cloud_llm_to_db(state.db_manager.as_ref(), Some(&cfg))
        .await
        .map_err(|e| e.to_string())?;
    state.cloud_llm_runtime.set_user_config(Some(cfg.clone()));
    Ok(to_resp(&state, Some(&cfg)))
}

#[tauri::command]
pub async fn verify_cloud_llm_ui_settings(state: State<'_, AppState>) -> Result<(), String> {
    let merged = resolve_cloud_llm_config(state.cloud_llm_runtime.as_ref()).ok_or_else(|| {
        "cloud_llm: nothing to verify (configure app settings or OCLIVE_CLOUD_LLM_* env)".to_string()
    })?;
    merged
        .probe_chat_minimal()
        .await
        .map_err(|e| e.to_string())
}

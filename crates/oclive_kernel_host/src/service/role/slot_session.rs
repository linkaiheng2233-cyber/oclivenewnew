//! Session slot override impls shared by HTTP routes and Tauri invoke.

use super::{get_role_info_impl, load_role_impl, session_namespace};
use crate::command_error::CommandError;
use crate::domain::role_snapshot::plugin_backends_override_from_slot_session;
use crate::error::AppError;
use crate::infrastructure::storage::resolve_llm_backend_env_override;
use crate::models::dto::{
    ClearAllSessionSlotOverridesRequest, ClearSessionSlotOverrideRequest,
    GetPluginResolutionDebugRequest, PluginResolutionDebugInfo, RoleInfo,
    SaveRoleSlotRegistryRequest, SetSessionPluginBackendRequest, SetSessionSlotOverrideRequest,
    API_VERSION, SCHEMA_VERSION,
};
use crate::models::plugin_backends::LlmBackend;
use crate::state::AppState;
use oclive_validation::{default_slot_key_for_module, SlotOverridePatch};
use serde_json::json;

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
pub async fn set_session_plugin_backend_impl(
    state: &AppState,
    req: &SetSessionPluginBackendRequest,
) -> Result<RoleInfo, CommandError> {
    let module = req.module.trim().to_ascii_lowercase();
    if req.local_memory_provider_id.is_some() && module.as_str() != "memory" {
        return Err(AppError::InvalidParameter(
            "local_memory_provider_id only supports module=memory".to_string(),
        )
        .into());
    }
    let slot_key = default_slot_key_for_module(&module).ok_or_else(|| {
        CommandError::from(AppError::InvalidParameter(format!(
            "session backend override: unknown module {}",
            req.module
        )))
    })?;
    let role = state.load_role_cached_async(&req.role_id).await?;
    if role.slot_registry.is_none() {
        return Err(AppError::InvalidParameter(
            "v2 slot_registry required; run `oclive pack migrate-to-blueprint` on the role pack"
                .to_string(),
        )
        .into());
    }
    let ns = session_namespace(&req.role_id, req.session_id.as_deref());
    if matches!(req.backend.as_ref(), Some(None)) && req.local_memory_provider_id.is_none() {
        state.clear_session_slot_override(ns.as_str(), slot_key);
        return get_role_info_impl(state, &req.role_id, req.session_id.as_deref()).await;
    }
    set_session_slot_override_impl(
        state,
        &SetSessionSlotOverrideRequest {
            role_id: req.role_id.clone(),
            slot_key: slot_key.to_string(),
            backend: req.backend.as_ref().and_then(|o| o.clone()),
            plugin: None,
            plugins: None,
            model: None,
            local_memory_provider_id: req.local_memory_provider_id.clone(),
            session_id: req.session_id.clone(),
        },
    )
    .await
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
pub async fn set_session_slot_override_impl(
    state: &AppState,
    req: &SetSessionSlotOverrideRequest,
) -> Result<RoleInfo, CommandError> {
    let slot_key = req.slot_key.trim();
    if slot_key.is_empty() {
        return Err(AppError::InvalidParameter("slot_key must not be empty".into()).into());
    }
    state.load_role_cached_async(&req.role_id).await?;
    let ns = session_namespace(&req.role_id, req.session_id.as_deref());
    state.db_manager.ensure_role_runtime(ns.as_str()).await?;

    let patch = SlotOverridePatch {
        backend: req.backend.clone(),
        plugin: req.plugin.clone(),
        plugins: req.plugins.clone(),
        model: req.model.clone(),
        local_memory_provider_id: req.local_memory_provider_id.clone(),
    };
    if patch.is_empty() {
        state.clear_session_slot_override(ns.as_str(), slot_key);
    } else {
        state.set_session_slot_override(ns.as_str(), slot_key, patch);
    }
    get_role_info_impl(state, &req.role_id, req.session_id.as_deref()).await
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
pub async fn clear_session_slot_override_impl(
    state: &AppState,
    req: &ClearSessionSlotOverrideRequest,
) -> Result<RoleInfo, CommandError> {
    let ns = session_namespace(&req.role_id, req.session_id.as_deref());
    state.clear_session_slot_override(ns.as_str(), req.slot_key.trim());
    get_role_info_impl(state, &req.role_id, req.session_id.as_deref()).await
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
pub async fn clear_all_session_slot_overrides_impl(
    state: &AppState,
    req: &ClearAllSessionSlotOverridesRequest,
) -> Result<RoleInfo, CommandError> {
    let ns = session_namespace(&req.role_id, req.session_id.as_deref());
    state.clear_all_session_slot_overrides(ns.as_str());
    get_role_info_impl(state, &req.role_id, req.session_id.as_deref()).await
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
pub async fn save_role_slot_registry_impl(
    state: &AppState,
    req: &SaveRoleSlotRegistryRequest,
) -> Result<RoleInfo, CommandError> {
    let role_id = req.role_id.trim();
    if role_id.is_empty() {
        return Err(AppError::InvalidParameter("role_id must not be empty".into()).into());
    }
    state
        .storage
        .save_blueprint_v2_slot_registry(role_id, &req.slot_registry)?;
    state.invalidate_role_cache(role_id);
    state.invalidate_personality_cache_for_role(role_id);
    load_role_impl(state, role_id, false).await?;
    get_role_info_impl(state, role_id, None).await
}

/// Writes `author.json` → `suggested_plugin_backends` into session-namespace backend override (does not write back to role pack).
///
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
pub async fn apply_author_suggested_plugin_backends_impl(
    state: &AppState,
    role_id: &str,
    session_id: Option<&str>,
) -> Result<RoleInfo, CommandError> {
    let role_id = role_id.trim();
    if role_id.is_empty() {
        return Err(AppError::InvalidParameter("role_id required".into()).into());
    }
    let role = state.storage.load_role(role_id)?;
    let Some(sugg) = role
        .author_pack
        .as_ref()
        .and_then(|a| a.suggested_plugin_backends.as_ref())
        .cloned()
    else {
        return Err(AppError::InvalidParameter(
            "This role pack has no author.json suggested_plugin_backends.".into(),
        )
        .into());
    };
    let ns = session_namespace(role_id, session_id);
    state.db_manager.ensure_role_runtime(ns.as_str()).await?;
    let role_cached = state.load_role_cached_async(role_id).await?;
    let Some(reg) = role_cached.slot_registry.as_ref() else {
        return Err(AppError::InvalidParameter(
            "v2 slot_registry required to apply author suggested backends".into(),
        )
        .into());
    };
    state.clear_all_session_slot_overrides(ns.as_str());
    let wire = |v: serde_json::Value, fallback: &str| -> String {
        v.as_str()
            .map(str::to_string)
            .unwrap_or_else(|| fallback.to_string())
    };
    let slots: [(&str, String); 6] = [
        (
            "memory",
            wire(
                serde_json::to_value(sugg.memory).unwrap_or(json!("builtin")),
                "builtin",
            ),
        ),
        (
            "emotion",
            wire(
                serde_json::to_value(sugg.emotion).unwrap_or(json!("builtin")),
                "builtin",
            ),
        ),
        (
            "event",
            wire(
                serde_json::to_value(sugg.event).unwrap_or(json!("builtin")),
                "builtin",
            ),
        ),
        (
            "prompt",
            wire(
                serde_json::to_value(sugg.prompt).unwrap_or(json!("builtin")),
                "builtin",
            ),
        ),
        (
            "llm",
            wire(
                serde_json::to_value(sugg.llm).unwrap_or(json!("ollama")),
                "ollama",
            ),
        ),
        (
            "agent",
            wire(
                serde_json::to_value(sugg.agent).unwrap_or(json!("builtin")),
                "builtin",
            ),
        ),
    ];
    for (module, backend) in slots {
        let Some(key) = default_slot_key_for_module(module) else {
            continue;
        };
        if !reg.contains_key(key) {
            continue;
        }
        let mut patch = SlotOverridePatch {
            backend: Some(backend),
            ..Default::default()
        };
        if module == "memory" {
            patch.local_memory_provider_id = sugg.local_memory_provider_id.clone();
        }
        state.set_session_slot_override(ns.as_str(), key, patch);
    }
    get_role_info_impl(state, role_id, session_id).await
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
pub async fn get_plugin_resolution_debug_impl(
    state: &AppState,
    req: &GetPluginResolutionDebugRequest,
) -> Result<PluginResolutionDebugInfo, CommandError> {
    build_plugin_resolution_debug_info(state, &req.role_id, req.session_id.as_deref()).await
}

pub async fn build_plugin_resolution_debug_info(
    state: &AppState,
    role_id: &str,
    session_id: Option<&str>,
) -> Result<PluginResolutionDebugInfo, CommandError> {
    let role = state.load_role_cached_async(role_id).await?;
    let session_ns = session_namespace(role_id, session_id);
    state
        .db_manager
        .ensure_role_runtime(session_ns.as_str())
        .await?;
    let session_override =
        plugin_backends_override_from_slot_session(state, role.as_ref(), session_ns.as_str());
    let effective = state.effective_plugin_backends_for_session(role.as_ref(), session_ns.as_str());
    let effective_sources =
        state.effective_plugin_backend_sources_for_session(role.as_ref(), session_ns.as_str());
    let llm_env_override = resolve_llm_backend_env_override().map(|b| match b {
        LlmBackend::Ollama => "ollama".to_string(),
        LlmBackend::Remote => "remote".to_string(),
        LlmBackend::Directory => "directory".to_string(),
    });
    let remote_plugin_url_configured = std::env::var("OCLIVE_REMOTE_PLUGIN_URL")
        .ok()
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false);
    let remote_llm_url_configured = std::env::var("OCLIVE_REMOTE_LLM_URL")
        .ok()
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false);
    let mut local_provider_ids: Vec<String> = state
        .local_plugin_all_providers()
        .iter()
        .map(|d| d.provider_id.clone())
        .collect();
    local_provider_ids.sort();
    local_provider_ids.dedup();

    Ok(PluginResolutionDebugInfo {
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        api_version: API_VERSION,
        schema_version: SCHEMA_VERSION,
        role_id: role_id.to_string(),
        session_namespace: session_ns,
        plugin_backends_pack_default: role.plugin_backends.as_ref().clone(),
        plugin_backends_session_override: session_override,
        plugin_backends_effective: effective.as_ref().clone(),
        plugin_backends_effective_sources: effective_sources,
        llm_env_override,
        remote_plugin_url_configured,
        remote_llm_url_configured,
        local_provider_count: local_provider_ids.len(),
        local_provider_ids,
    })
}

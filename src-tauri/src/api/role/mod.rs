//! 角色 API：清单加载、运行时快照、身份与进化系数等 Tauri 命令。

mod display;
mod interaction;
mod runtime;

use crate::error::AppError;
use crate::infrastructure::storage::resolve_llm_backend_env_override;
use crate::models::dto::{
    ClearSceneUserRelationRequest, GetPluginResolutionDebugRequest, GetRoleInfoRequest,
    PluginResolutionDebugInfo, RoleData, RoleInfo, RoleSummary, SceneLabelEntry,
    SetEvolutionFactorRequest, SetRemoteLifeEnabledRequest, SetRoleInteractionModeRequest,
    SetSceneUserRelationRequest, SetSessionPluginBackendRequest, SetUserRelationRequest,
    API_VERSION, OCLIVE_DEFAULT_RELATION_SENTINEL, SCHEMA_VERSION,
};
use crate::models::plugin_backends::{
    EmotionBackend, EventBackend, LlmBackend, MemoryBackend, PluginBackendsOverride, PromptBackend,
};
use crate::models::role::IdentityBinding;
use crate::state::AppState;
use std::sync::Arc;
use tauri::State;

use interaction::resolve_interaction_ui_snapshot;
use runtime::{
    current_favorability_for_effective_identity, maybe_seed_initial_favorability_with_extras,
    resolve_relation_state_for_ui, role_runtime_extras,
};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};

const EVENT_IMPACT_MIN: f64 = 0.05;
const EVENT_IMPACT_MAX: f64 = 5.0;

pub(crate) fn session_namespace(role_id: &str, session_id: Option<&str>) -> String {
    crate::domain::chat_engine::conversation_state_role_id(role_id, session_id)
}

fn parse_backend_wire<T: DeserializeOwned>(module: &str, value: &str) -> Result<T, String> {
    let t = value.trim();
    if t.is_empty() {
        return Err(AppError::InvalidParameter(format!(
            "session backend override: module={} backend 不能为空",
            module
        ))
        .to_frontend_error());
    }
    serde_json::from_value::<T>(Value::String(t.to_string())).map_err(|_| {
        AppError::InvalidParameter(format!(
            "session backend override: module={} backend={} 非法",
            module, t
        ))
        .to_frontend_error()
    })
}

/// `reset_portrait_emotion`：为 `true` 时（应用启动 `load_role`）立绘重置为 `neutral`；切换角色时为 `false` 以保留各角色上次立绘状态。
pub async fn load_role_impl(
    state: &AppState,
    role_id: &str,
    reset_portrait_emotion: bool,
) -> Result<RoleData, String> {
    let role = state
        .storage
        .load_role(role_id)
        .map_err(|e| e.to_frontend_error())?;
    let role = Arc::new(role);

    state.directory_plugins.set_active_role_id(role_id);
    state
        .directory_plugins
        .ensure_role_plugin_state(role_id, role.plugin_state_ui_baseline());

    state.invalidate_personality_cache_for_role(role_id);

    state
        .db_manager
        .ensure_role_runtime(role_id)
        .await
        .map_err(|e| e.to_frontend_error())?;

    if reset_portrait_emotion {
        state
            .db_manager
            .set_current_emotion(role_id, "neutral")
            .await
            .map_err(|e| e.to_frontend_error())?;
    }

    let personality = state
        .get_current_personality(role_id, role.as_ref())
        .await
        .map_err(|e| e.to_frontend_error())?;

    let current_scene = state
        .db_manager
        .get_current_scene(role_id)
        .await
        .map_err(|e| e.to_frontend_error())?;
    let rt = role_runtime_extras(state, role_id, current_scene.as_deref(), role.as_ref()).await?;
    maybe_seed_initial_favorability_with_extras(state, role_id, role.as_ref(), &rt).await?;
    let current_favorability = current_favorability_for_effective_identity(
        state,
        role_id,
        rt.current_user_relation.as_str(),
    )
    .await?;

    let memory_count = state
        .memory_repo
        .count_memories(role_id)
        .await
        .map_err(|e| e.to_frontend_error())?;

    let event_count = state
        .db_manager
        .count_events(role_id)
        .await
        .map_err(|e| e.to_frontend_error())?;
    let effective_ollama_model = role.resolve_ollama_model(state.ollama_model.as_str());
    let relation_state =
        resolve_relation_state_for_ui(state, role_id, rt.current_user_relation.as_str()).await?;
    let remote_life_enabled = state
        .db_manager
        .get_remote_life_enabled(role_id)
        .await
        .map_err(|e| e.to_frontend_error())?;
    let remote_life_pack_default = role
        .remote_presence
        .as_ref()
        .and_then(|r| r.default_enabled);

    let virtual_time_ms = state
        .db_manager
        .get_virtual_time_ms(role_id)
        .await
        .map_err(|e| e.to_frontend_error())?
        .unwrap_or(0);
    let interaction =
        resolve_interaction_ui_snapshot(state, role_id, role.as_ref(), virtual_time_ms).await?;

    state
        .role_cache
        .write()
        .insert(role_id.to_string(), Arc::clone(&role));
    let session_ns = session_namespace(role_id, None);
    let plugin_backends_session_override = state.session_backend_override(session_ns.as_str());
    let plugin_backends_effective =
        state.effective_plugin_backends_for_session(role.as_ref(), session_ns.as_str());
    let plugin_backends_effective_sources =
        state.effective_plugin_backend_sources_for_session(session_ns.as_str());

    Ok(RoleData {
        role_id: role_id.to_string(),
        name: role.name.clone(),
        version: role.version.clone(),
        author: role.author.clone(),
        description: role.description.clone(),
        personality_vector: personality.to_vec7(),
        current_favorability,
        current_emotion: state
            .db_manager
            .get_current_emotion(role_id)
            .await
            .map_err(|e| e.to_frontend_error())?
            .unwrap_or_else(|| "Neutral".to_string()),
        memory_count: memory_count as i32,
        event_count: event_count as i32,
        user_relations: rt.user_relations,
        default_relation: rt.default_relation,
        relation_state,
        current_user_relation: rt.current_user_relation.clone(),
        use_manifest_default: rt.use_manifest_default,
        remote_life_enabled,
        remote_life_pack_default,
        event_impact_factor: rt.event_impact_factor,
        personality_source: role.evolution_config.personality_source,
        effective_ollama_model,
        identity_binding: role.identity_binding,
        interaction_mode: interaction.mode_str,
        interaction_mode_pack_default: interaction.pack_default,
        current_life: interaction.current_life,
        plugin_backends: role.plugin_backends.clone(),
        plugin_backends_session_override,
        plugin_backends_effective,
        plugin_backends_effective_sources,
        pack_ui_config: role.ui_config.clone(),
        pack_ui_baseline: role.plugin_state_ui_baseline().clone(),
        author_pack: role.author_pack.clone(),
    })
}

pub async fn get_role_info_impl(
    state: &AppState,
    role_id: &str,
    session_id: Option<&str>,
) -> Result<RoleInfo, String> {
    let session_ns = session_namespace(role_id, session_id);
    if !state
        .db_manager
        .role_runtime_exists(session_ns.as_str())
        .await
        .map_err(|e| e.to_frontend_error())?
    {
        return Err(AppError::InvalidParameter(
            "Role runtime not initialized; call load_role first".to_string(),
        )
        .to_frontend_error());
    }

    let role = state
        .load_role_cached(role_id)
        .map_err(|e| e.to_frontend_error())?;
    let plugin_backends_session_override = state.session_backend_override(session_ns.as_str());
    let plugin_backends_effective =
        state.effective_plugin_backends_for_session(role.as_ref(), session_ns.as_str());
    let plugin_backends_effective_sources =
        state.effective_plugin_backend_sources_for_session(session_ns.as_str());

    let current_scene = state
        .db_manager
        .get_current_scene(role_id)
        .await
        .map_err(|e| e.to_frontend_error())?;
    let user_presence_scene = state
        .db_manager
        .get_user_presence_scene(role_id)
        .await
        .map_err(|e| e.to_frontend_error())?;
    let rt = role_runtime_extras(state, role_id, current_scene.as_deref(), role.as_ref()).await?;
    maybe_seed_initial_favorability_with_extras(state, role_id, role.as_ref(), &rt).await?;

    let personality = state
        .get_current_personality(role_id, role.as_ref())
        .await
        .map_err(|e| e.to_frontend_error())?;

    let last_interaction = state
        .db_manager
        .get_latest_memory_created_at(role_id)
        .await
        .map_err(|e| e.to_frontend_error())?
        .map(|t| t.to_rfc3339());

    let scenes = state
        .storage
        .list_scene_ids(role_id)
        .map_err(|e| e.to_frontend_error())?;
    let scene_labels: Vec<SceneLabelEntry> = scenes
        .iter()
        .map(|id| SceneLabelEntry {
            id: id.clone(),
            label: state.storage.scene_display_name(role_id, id),
        })
        .collect();
    let virtual_time_ms = state
        .db_manager
        .get_virtual_time_ms(role_id)
        .await
        .map_err(|e| e.to_frontend_error())?
        .unwrap_or(0);
    let current_favorability = current_favorability_for_effective_identity(
        state,
        role_id,
        rt.current_user_relation.as_str(),
    )
    .await?;
    let effective_ollama_model = role.resolve_ollama_model(state.ollama_model.as_str());
    let relation_state =
        resolve_relation_state_for_ui(state, role_id, rt.current_user_relation.as_str()).await?;
    let remote_life_enabled = state
        .db_manager
        .get_remote_life_enabled(role_id)
        .await
        .map_err(|e| e.to_frontend_error())?;
    let remote_life_pack_default = role
        .remote_presence
        .as_ref()
        .and_then(|r| r.default_enabled);

    let interaction =
        resolve_interaction_ui_snapshot(state, role_id, role.as_ref(), virtual_time_ms).await?;

    let (knowledge_enabled, knowledge_chunk_count) = match &role.knowledge_index {
        Some(idx) => (true, idx.chunks.len() as i32),
        None => (false, 0),
    };

    Ok(RoleInfo {
        role_id: role_id.to_string(),
        role_name: role.name.clone(),
        version: role.version.clone(),
        author: role.author.clone(),
        description: role.description.clone(),
        current_favorability,
        current_emotion: state
            .db_manager
            .get_current_emotion(role_id)
            .await
            .map_err(|e| e.to_frontend_error())?
            .unwrap_or_else(|| "Neutral".to_string()),
        personality_vector: personality.to_vec7(),
        personality_source: role.evolution_config.personality_source,
        last_interaction,
        scenes,
        scene_labels,
        current_scene,
        user_presence_scene,
        virtual_time_ms,
        user_relations: rt.user_relations,
        default_relation: rt.default_relation,
        current_user_relation: rt.current_user_relation.clone(),
        use_manifest_default: rt.use_manifest_default,
        relation_state,
        remote_life_enabled,
        remote_life_pack_default,
        event_impact_factor: rt.event_impact_factor,
        effective_ollama_model,
        identity_binding: role.identity_binding,
        interaction_mode: interaction.mode_str,
        interaction_mode_pack_default: interaction.pack_default,
        current_life: interaction.current_life,
        plugin_backends: role.plugin_backends.clone(),
        plugin_backends_session_override,
        plugin_backends_effective,
        plugin_backends_effective_sources,
        knowledge_enabled,
        knowledge_chunk_count,
        pack_ui_config: role.ui_config.clone(),
        pack_ui_baseline: role.plugin_state_ui_baseline().clone(),
        author_pack: role.author_pack.clone(),
    })
}

pub async fn list_roles_impl(state: &AppState) -> Result<Vec<RoleSummary>, String> {
    let list_dev = crate::env_flags::list_dev_roles_enabled();
    let roles = state
        .storage
        .load_all_roles()
        .map_err(|e| e.to_frontend_error())?;
    Ok(roles
        .into_iter()
        .filter(|r| list_dev || !r.dev_only)
        .map(|r| RoleSummary {
            id: r.id,
            name: r.name,
            version: r.version,
            author: r.author,
        })
        .collect())
}

pub async fn switch_role_impl(state: &AppState, role_id: &str) -> Result<RoleInfo, String> {
    load_role_impl(state, role_id, false).await?;
    get_role_info_impl(state, role_id, None).await
}

#[tauri::command]
pub async fn load_role(role_id: String, state: State<'_, AppState>) -> Result<RoleData, String> {
    load_role_impl(&state, &role_id, true).await
}

#[tauri::command]
pub async fn get_role_info(
    req: GetRoleInfoRequest,
    state: State<'_, AppState>,
) -> Result<RoleInfo, String> {
    get_role_info_impl(&state, &req.role_id, req.session_id.as_deref()).await
}

#[tauri::command]
pub async fn list_roles(state: State<'_, AppState>) -> Result<Vec<RoleSummary>, String> {
    list_roles_impl(&state).await
}

#[tauri::command]
pub async fn switch_role(role_id: String, state: State<'_, AppState>) -> Result<RoleInfo, String> {
    switch_role_impl(&state, &role_id).await
}

pub async fn set_user_relation_impl(
    state: &AppState,
    req: &SetUserRelationRequest,
) -> Result<RoleInfo, String> {
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
        .load_role_cached(&req.role_id)
        .map_err(|e| e.to_frontend_error())?;

    if matches!(role.identity_binding, IdentityBinding::Global) {
        state
            .db_manager
            .clear_all_scene_identities_for_role(&req.role_id)
            .await
            .map_err(|e| e.to_frontend_error())?;
    }

    if req.relation == OCLIVE_DEFAULT_RELATION_SENTINEL {
        state
            .db_manager
            .set_use_manifest_default(&req.role_id, true)
            .await
            .map_err(|e| e.to_frontend_error())?;
        let eff = role.default_relation.clone();
        let seed = role.initial_favorability_for_relation(eff.as_str());
        state
            .db_manager
            .ensure_identity_stats_row(&req.role_id, &eff, seed)
            .await
            .map_err(|e| e.to_frontend_error())?;
        state
            .db_manager
            .mirror_runtime_from_identity(&req.role_id, &eff)
            .await
            .map_err(|e| e.to_frontend_error())?;
        return get_role_info_impl(state, &req.role_id, None).await;
    }

    if !role.user_relations.iter().any(|r| r.id == req.relation) {
        return Err(
            AppError::InvalidParameter(format!("unknown relation: {}", req.relation))
                .to_frontend_error(),
        );
    }
    state
        .db_manager
        .set_use_manifest_default(&req.role_id, false)
        .await
        .map_err(|e| e.to_frontend_error())?;
    state
        .db_manager
        .set_user_relation(&req.role_id, &req.relation)
        .await
        .map_err(|e| e.to_frontend_error())?;
    let seed = role.initial_favorability_for_relation(&req.relation);
    state
        .db_manager
        .ensure_identity_stats_row(&req.role_id, &req.relation, seed)
        .await
        .map_err(|e| e.to_frontend_error())?;
    state
        .db_manager
        .mirror_runtime_from_identity(&req.role_id, &req.relation)
        .await
        .map_err(|e| e.to_frontend_error())?;
    get_role_info_impl(state, &req.role_id, None).await
}

pub async fn set_evolution_factor_impl(
    state: &AppState,
    req: &SetEvolutionFactorRequest,
) -> Result<RoleInfo, String> {
    let f = req.event_impact_factor;
    if !f.is_finite() || !(EVENT_IMPACT_MIN..=EVENT_IMPACT_MAX).contains(&f) {
        return Err(AppError::InvalidParameter(format!(
            "event_impact_factor must be between {} and {}",
            EVENT_IMPACT_MIN, EVENT_IMPACT_MAX
        ))
        .to_frontend_error());
    }
    state
        .load_role_cached(&req.role_id)
        .map_err(|e| e.to_frontend_error())?;
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
    state
        .db_manager
        .set_event_impact_factor(&req.role_id, f)
        .await
        .map_err(|e| e.to_frontend_error())?;
    get_role_info_impl(state, &req.role_id, None).await
}

pub async fn clear_scene_user_relation_impl(
    state: &AppState,
    req: &ClearSceneUserRelationRequest,
) -> Result<RoleInfo, String> {
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
        .load_role_cached(&req.role_id)
        .map_err(|e| e.to_frontend_error())?;
    if matches!(role.identity_binding, IdentityBinding::Global) {
        return Err(AppError::InvalidParameter(
            "当前角色包为全局身份模式（identity_binding: global），无需按场景清除身份覆盖"
                .to_string(),
        )
        .to_frontend_error());
    }
    let scenes = state
        .storage
        .list_scene_ids(&req.role_id)
        .map_err(|e| e.to_frontend_error())?;
    if !scenes.iter().any(|s| s == &req.scene_id) {
        return Err(AppError::InvalidParameter(format!(
            "scene_id not in role pack: {}",
            req.scene_id
        ))
        .to_frontend_error());
    }
    state
        .db_manager
        .clear_user_relation_for_scene(&req.role_id, &req.scene_id)
        .await
        .map_err(|e| e.to_frontend_error())?;
    get_role_info_impl(state, &req.role_id, None).await
}

pub async fn set_scene_user_relation_impl(
    state: &AppState,
    req: &SetSceneUserRelationRequest,
) -> Result<RoleInfo, String> {
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
        .load_role_cached(&req.role_id)
        .map_err(|e| e.to_frontend_error())?;
    if matches!(role.identity_binding, IdentityBinding::Global) {
        return Err(AppError::InvalidParameter(
            "当前角色包为全局身份模式（identity_binding: global），请使用全局身份设置，勿按场景绑定".to_string(),
        )
        .to_frontend_error());
    }
    if !role.user_relations.iter().any(|r| r.id == req.relation) {
        return Err(
            AppError::InvalidParameter(format!("unknown relation: {}", req.relation))
                .to_frontend_error(),
        );
    }
    let scenes = state
        .storage
        .list_scene_ids(&req.role_id)
        .map_err(|e| e.to_frontend_error())?;
    if !scenes.iter().any(|s| s == &req.scene_id) {
        return Err(AppError::InvalidParameter(format!(
            "scene_id not in role pack: {}",
            req.scene_id
        ))
        .to_frontend_error());
    }
    state
        .db_manager
        .set_use_manifest_default(&req.role_id, false)
        .await
        .map_err(|e| e.to_frontend_error())?;
    state
        .db_manager
        .set_user_relation_for_scene(&req.role_id, &req.scene_id, &req.relation)
        .await
        .map_err(|e| e.to_frontend_error())?;
    get_role_info_impl(state, &req.role_id, None).await
}

#[tauri::command]
pub async fn set_user_relation(
    req: SetUserRelationRequest,
    state: State<'_, AppState>,
) -> Result<RoleInfo, String> {
    set_user_relation_impl(&state, &req).await
}

#[tauri::command]
pub async fn set_evolution_factor(
    req: SetEvolutionFactorRequest,
    state: State<'_, AppState>,
) -> Result<RoleInfo, String> {
    set_evolution_factor_impl(&state, &req).await
}

pub async fn set_remote_life_enabled_impl(
    state: &AppState,
    req: &SetRemoteLifeEnabledRequest,
) -> Result<RoleInfo, String> {
    state
        .load_role_cached(&req.role_id)
        .map_err(|e| e.to_frontend_error())?;
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
    state
        .db_manager
        .set_remote_life_enabled(&req.role_id, req.enabled)
        .await
        .map_err(|e| e.to_frontend_error())?;
    get_role_info_impl(state, &req.role_id, None).await
}

#[tauri::command]
pub async fn set_remote_life_enabled(
    req: SetRemoteLifeEnabledRequest,
    state: State<'_, AppState>,
) -> Result<RoleInfo, String> {
    set_remote_life_enabled_impl(&state, &req).await
}

pub async fn set_role_interaction_mode_impl(
    state: &AppState,
    req: &SetRoleInteractionModeRequest,
) -> Result<RoleInfo, String> {
    state
        .load_role_cached(&req.role_id)
        .map_err(|e| e.to_frontend_error())?;
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
    state
        .db_manager
        .set_interaction_mode_for_role(&req.role_id, req.mode.trim())
        .await
        .map_err(|e| e.to_frontend_error())?;
    get_role_info_impl(state, &req.role_id, None).await
}

#[tauri::command]
pub async fn set_role_interaction_mode(
    req: SetRoleInteractionModeRequest,
    state: State<'_, AppState>,
) -> Result<RoleInfo, String> {
    set_role_interaction_mode_impl(&state, &req).await
}

pub async fn set_session_plugin_backend_impl(
    state: &AppState,
    req: &SetSessionPluginBackendRequest,
) -> Result<RoleInfo, String> {
    state
        .load_role_cached(&req.role_id)
        .map_err(|e| e.to_frontend_error())?;
    let ns = session_namespace(&req.role_id, req.session_id.as_deref());
    state
        .db_manager
        .ensure_role_runtime(ns.as_str())
        .await
        .map_err(|e| e.to_frontend_error())?;
    let mut next = state
        .session_backend_override(ns.as_str())
        .unwrap_or_default();
    let module = req.module.trim().to_ascii_lowercase();
    if req.local_memory_provider_id.is_some() && module.as_str() != "memory" {
        return Err(AppError::InvalidParameter(
            "local_memory_provider_id only supports module=memory".to_string(),
        )
        .to_frontend_error());
    }
    match module.as_str() {
        "memory" => {
            if let Some(backend) = req.backend.as_ref() {
                next.memory = backend
                    .as_deref()
                    .map(|v| parse_backend_wire::<MemoryBackend>("memory", v))
                    .transpose()?;
            }
            if let Some(provider_id) = req.local_memory_provider_id.as_ref() {
                let t = provider_id.trim();
                if t.is_empty() {
                    next.local_memory_provider_id = None;
                } else {
                    next.local_memory_provider_id = Some(t.to_string());
                }
            }
        }
        "emotion" => {
            if let Some(backend) = req.backend.as_ref() {
                next.emotion = backend
                    .as_deref()
                    .map(|v| parse_backend_wire::<EmotionBackend>("emotion", v))
                    .transpose()?;
            }
        }
        "event" => {
            if let Some(backend) = req.backend.as_ref() {
                next.event = backend
                    .as_deref()
                    .map(|v| parse_backend_wire::<EventBackend>("event", v))
                    .transpose()?;
            }
        }
        "prompt" => {
            if let Some(backend) = req.backend.as_ref() {
                next.prompt = backend
                    .as_deref()
                    .map(|v| parse_backend_wire::<PromptBackend>("prompt", v))
                    .transpose()?;
            }
        }
        "llm" => {
            if let Some(backend) = req.backend.as_ref() {
                next.llm = backend
                    .as_deref()
                    .map(|v| parse_backend_wire::<LlmBackend>("llm", v))
                    .transpose()?;
            }
        }
        _ => {
            return Err(AppError::InvalidParameter(format!(
                "session backend override: unknown module {}",
                req.module
            ))
            .to_frontend_error());
        }
    }
    if next.is_empty() {
        state.clear_session_backend_override(ns.as_str());
    } else {
        state.set_session_backend_override(ns.as_str(), next);
    }
    get_role_info_impl(state, &req.role_id, req.session_id.as_deref()).await
}

#[tauri::command]
pub async fn set_session_plugin_backend(
    req: SetSessionPluginBackendRequest,
    state: State<'_, AppState>,
) -> Result<RoleInfo, String> {
    set_session_plugin_backend_impl(&state, &req).await
}

#[derive(Debug, serde::Deserialize)]
pub struct ApplyAuthorSuggestedBackendsRequest {
    pub role_id: String,
    #[serde(default)]
    pub session_id: Option<String>,
}

/// 将 `author.json` → `suggested_plugin_backends` 写入当前会话命名空间的后端覆盖（不写回角色包）。
#[tauri::command]
pub async fn apply_author_suggested_plugin_backends(
    req: ApplyAuthorSuggestedBackendsRequest,
    state: State<'_, AppState>,
) -> Result<RoleInfo, String> {
    let role_id = req.role_id.trim();
    if role_id.is_empty() {
        return Err(AppError::InvalidParameter("role_id required".into()).to_frontend_error());
    }
    let role = state
        .storage
        .load_role(role_id)
        .map_err(|e| e.to_frontend_error())?;
    let Some(sugg) = role
        .author_pack
        .as_ref()
        .and_then(|a| a.suggested_plugin_backends.as_ref())
        .cloned()
    else {
        return Err(AppError::InvalidParameter(
            "该角色包未提供 author.json suggested_plugin_backends".into(),
        )
        .to_frontend_error());
    };
    let ns = session_namespace(role_id, req.session_id.as_deref());
    state
        .db_manager
        .ensure_role_runtime(ns.as_str())
        .await
        .map_err(|e| e.to_frontend_error())?;
    let ov = PluginBackendsOverride {
        memory: Some(sugg.memory),
        emotion: Some(sugg.emotion),
        event: Some(sugg.event),
        prompt: Some(sugg.prompt),
        llm: Some(sugg.llm),
        local_memory_provider_id: sugg.local_memory_provider_id.clone(),
        directory_plugins: Some(sugg.directory_plugins.clone()),
    };
    state.set_session_backend_override(ns.as_str(), ov);
    get_role_info_impl(&state, role_id, req.session_id.as_deref()).await
}

pub async fn get_plugin_resolution_debug_impl(
    state: &AppState,
    req: &GetPluginResolutionDebugRequest,
) -> Result<PluginResolutionDebugInfo, String> {
    build_plugin_resolution_debug_info(state, &req.role_id, req.session_id.as_deref()).await
}

#[tauri::command]
pub async fn get_plugin_resolution_debug(
    req: GetPluginResolutionDebugRequest,
    state: State<'_, AppState>,
) -> Result<PluginResolutionDebugInfo, String> {
    get_plugin_resolution_debug_impl(&state, &req).await
}

pub(crate) async fn build_plugin_resolution_debug_info(
    state: &AppState,
    role_id: &str,
    session_id: Option<&str>,
) -> Result<PluginResolutionDebugInfo, String> {
    let role = state
        .load_role_cached(role_id)
        .map_err(|e| e.to_frontend_error())?;
    let session_ns = session_namespace(role_id, session_id);
    state
        .db_manager
        .ensure_role_runtime(session_ns.as_str())
        .await
        .map_err(|e| e.to_frontend_error())?;
    let session_override = state.session_backend_override(session_ns.as_str());
    let effective = state.effective_plugin_backends_for_session(role.as_ref(), session_ns.as_str());
    let effective_sources = state.effective_plugin_backend_sources_for_session(session_ns.as_str());
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
        plugin_backends_pack_default: role.plugin_backends.clone(),
        plugin_backends_session_override: session_override,
        plugin_backends_effective: effective,
        plugin_backends_effective_sources: effective_sources,
        llm_env_override,
        remote_plugin_url_configured,
        remote_llm_url_configured,
        local_provider_count: local_provider_ids.len(),
        local_provider_ids,
    })
}

#[tauri::command]
pub async fn set_scene_user_relation(
    req: SetSceneUserRelationRequest,
    state: State<'_, AppState>,
) -> Result<RoleInfo, String> {
    set_scene_user_relation_impl(&state, &req).await
}

#[tauri::command]
pub async fn clear_scene_user_relation(
    req: ClearSceneUserRelationRequest,
    state: State<'_, AppState>,
) -> Result<RoleInfo, String> {
    clear_scene_user_relation_impl(&state, &req).await
}

/// 删除本地角色目录及该 manifest 角色（含 `__sess__` 会话命名空间）的 DB 状态。
pub async fn delete_role_impl(state: &AppState, role_id: String) -> Result<Value, String> {
    let rid = role_id.trim();
    if rid.is_empty() {
        return Err("delete_role: role_id required".to_string());
    }
    let removed_ns = state
        .db_manager
        .delete_all_data_for_manifest_role(rid)
        .await
        .map_err(|e| e.to_frontend_error())?;
    for ns in &removed_ns {
        state.clear_session_backend_override(ns);
    }
    let dir = state.storage.roles_dir().join(rid);
    if dir.exists() {
        let dir_owned = dir.clone();
        tokio::task::spawn_blocking(move || std::fs::remove_dir_all(&dir_owned))
            .await
            .map_err(|e| format!("delete_role: join {e}"))?
            .map_err(|e: std::io::Error| e.to_string())?;
    }
    state.directory_plugins.remove_role_plugin_state(rid)?;
    state.role_cache.write().remove(rid);
    state.invalidate_personality_cache_for_role(rid);
    Ok(json!({ "ok": true, "role_id": rid }))
}

/// 去掉 Windows 冗长路径前缀 `\\?\`，避免前端路径异常。
fn path_string_for_frontend(p: &std::path::Path) -> String {
    let s = p.to_string_lossy();
    const VERBATIM: &str = "\\\\?\\";
    if let Some(stripped) = s.strip_prefix(VERBATIM) {
        stripped.to_string()
    } else {
        s.into_owned()
    }
}

/// 解析 `roles/{role_id}/{relative}` 的绝对路径；文件存在时供前端 `convertFileSrc` / `readBinaryFile` 加载。
#[tauri::command]
pub fn resolve_role_asset_path(
    role_id: String,
    relative: String,
    state: State<'_, AppState>,
) -> Option<String> {
    let p = state.storage.role_asset_path(&role_id, &relative);
    if p.is_file() {
        return Some(path_string_for_frontend(&p));
    }
    None
}

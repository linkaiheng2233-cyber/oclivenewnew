//! 角色 API：清单加载、运行时快照、身份与进化系数等 Tauri 命令。
//!
//! 蓝图 v2 写盘走 [`save_role_slot_registry`]；**无**仅写 `manifest.json`/`settings.json`
//! `plugin_backends` 的遗留 Tauri 命令（旧包仅 [`RoleStorage::load_role_from_legacy_manifest_dir`] 只读兼容）。

mod display;
mod interaction;
mod runtime;

use crate::error::AppError;
use crate::infrastructure::storage::resolve_llm_backend_env_override;
use crate::models::dto::{
    ClearAllSessionSlotOverridesRequest, ClearSceneUserRelationRequest,
    ClearSessionSlotOverrideRequest, GetPluginResolutionDebugRequest, GetRoleInfoRequest,
    PluginResolutionDebugInfo, RoleData, RoleInfo, RoleSummary, SaveRoleSlotRegistryRequest,
    SceneLabelEntry, SetEvolutionFactorRequest, SetRemoteLifeEnabledRequest,
    SetRoleInteractionModeRequest, SetSceneUserRelationRequest, SetSessionPluginBackendRequest,
    SetSessionSlotOverrideRequest, SetUserRelationRequest, API_VERSION,
    OCLIVE_DEFAULT_RELATION_SENTINEL, SCHEMA_VERSION,
};
use crate::models::plugin_backends::{
    AgentBackend, EmotionBackend, EventBackend, LlmBackend, MemoryBackend, PluginBackendsOverride,
    PromptBackend,
};
use crate::models::role::{IdentityBinding, Role};
use crate::state::AppState;
use oclive_validation::{default_slot_key_for_module, SlotOverridePatch};
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

type SlotRegistryMap = std::collections::BTreeMap<String, oclive_validation::SlotRegistryEntry>;

fn slot_registry_role_info_fields(
    state: &AppState,
    role: &Role,
    session_ns: &str,
) -> (
    Option<SlotRegistryMap>,
    Option<SlotRegistryMap>,
    Vec<String>,
) {
    let pack = role.slot_registry.clone();
    let effective = state.effective_slot_registry_for_session(role, session_ns);
    let keys = state.slot_session_overridden_keys(session_ns);
    (pack, effective, keys)
}

/// 从会话 `slot_registry` 覆盖折叠为 C1 兼容的六槽 `PluginBackendsOverride`（只读展示）。
fn plugin_backends_override_from_slot_session(
    state: &AppState,
    role: &Role,
    session_namespace: &str,
) -> Option<PluginBackendsOverride> {
    let pack = role.slot_registry.as_ref()?;
    let ov = state.session_slot_overrides(session_namespace);
    if ov.is_empty() {
        return None;
    }
    let mut out = PluginBackendsOverride::default();
    for (key, patch) in ov {
        let Some(entry) = pack.get(&key) else {
            continue;
        };
        if let Some(ref b) = patch.backend {
            let wire = b.trim();
            if wire.is_empty() {
                continue;
            }
            match entry.slot_type.as_str() {
                "memory" => {
                    out.memory = parse_backend_wire::<MemoryBackend>("memory", wire).ok();
                }
                "emotion" => {
                    out.emotion = parse_backend_wire::<EmotionBackend>("emotion", wire).ok();
                }
                "event" => {
                    out.event = parse_backend_wire::<EventBackend>("event", wire).ok();
                }
                "prompt" => {
                    out.prompt = parse_backend_wire::<PromptBackend>("prompt", wire).ok();
                }
                "llm" => {
                    out.llm = parse_backend_wire::<LlmBackend>("llm", wire).ok();
                }
                "agent" => {
                    out.agent = parse_backend_wire::<AgentBackend>("agent", wire).ok();
                }
                _ => {}
            }
        }
        if entry.slot_type == "memory" {
            if let Some(ref id) = patch.local_memory_provider_id {
                let t = id.trim();
                out.local_memory_provider_id = if t.is_empty() {
                    None
                } else {
                    Some(t.to_string())
                };
            }
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

fn parse_backend_wire<T: DeserializeOwned>(module: &str, value: &str) -> Result<T, String> {
    let t = value.trim();
    if t.is_empty() {
        return Err(AppError::InvalidParameter(format!(
            "session backend override: module={} backend must not be empty",
            module
        ))
        .to_frontend_error());
    }
    serde_json::from_value::<T>(Value::String(t.to_string())).map_err(|_| {
        AppError::InvalidParameter(format!(
            "session backend override: module={} backend={} is invalid",
            module, t
        ))
        .to_frontend_error()
    })
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
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
    let plugin_backends_session_override =
        plugin_backends_override_from_slot_session(state, role.as_ref(), session_ns.as_str());
    let plugin_backends_effective =
        state.effective_plugin_backends_for_session(role.as_ref(), session_ns.as_str());
    let plugin_backends_effective_sources =
        state.effective_plugin_backend_sources_for_session(role.as_ref(), session_ns.as_str());
    let (slot_registry_pack, slot_registry_effective, slot_session_overridden_keys) =
        slot_registry_role_info_fields(state, role.as_ref(), session_ns.as_str());

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
        slot_registry_pack,
        slot_registry_effective,
        slot_session_overridden_keys,
    })
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
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
        return Err(AppError::RoleRuntimeNotReady.to_frontend_error());
    }

    let role = state
        .load_role_cached(role_id)
        .map_err(|e| e.to_frontend_error())?;
    let plugin_backends_session_override =
        plugin_backends_override_from_slot_session(state, role.as_ref(), session_ns.as_str());
    let plugin_backends_effective =
        state.effective_plugin_backends_for_session(role.as_ref(), session_ns.as_str());
    let plugin_backends_effective_sources =
        state.effective_plugin_backend_sources_for_session(role.as_ref(), session_ns.as_str());

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

    let (slot_registry_pack, slot_registry_effective, slot_session_overridden_keys) =
        slot_registry_role_info_fields(state, role.as_ref(), session_ns.as_str());

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
        slot_registry_pack,
        slot_registry_effective,
        slot_session_overridden_keys,
    })
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
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
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
pub async fn switch_role_impl(state: &AppState, role_id: &str) -> Result<RoleInfo, String> {
    load_role_impl(state, role_id, false).await?;
    get_role_info_impl(state, role_id, None).await
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn load_role(role_id: String, state: State<'_, AppState>) -> Result<RoleData, String> {
    load_role_impl(&state, &role_id, true).await
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn get_role_info(
    req: GetRoleInfoRequest,
    state: State<'_, AppState>,
) -> Result<RoleInfo, String> {
    get_role_info_impl(&state, &req.role_id, req.session_id.as_deref()).await
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn list_roles(state: State<'_, AppState>) -> Result<Vec<RoleSummary>, String> {
    list_roles_impl(&state).await
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn switch_role(role_id: String, state: State<'_, AppState>) -> Result<RoleInfo, String> {
    switch_role_impl(&state, &role_id).await
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
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
        return Err(AppError::RoleRuntimeNotReady.to_frontend_error());
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
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
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
        return Err(AppError::RoleRuntimeNotReady.to_frontend_error());
    }
    state
        .db_manager
        .set_event_impact_factor(&req.role_id, f)
        .await
        .map_err(|e| e.to_frontend_error())?;
    get_role_info_impl(state, &req.role_id, None).await
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
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
        return Err(AppError::RoleRuntimeNotReady.to_frontend_error());
    }
    let role = state
        .load_role_cached(&req.role_id)
        .map_err(|e| e.to_frontend_error())?;
    if matches!(role.identity_binding, IdentityBinding::Global) {
        return Err(AppError::InvalidParameter(
            "This role pack uses global identity_binding; per-scene identity overrides are not used."
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
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
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
        return Err(AppError::RoleRuntimeNotReady.to_frontend_error());
    }
    let role = state
        .load_role_cached(&req.role_id)
        .map_err(|e| e.to_frontend_error())?;
    if matches!(role.identity_binding, IdentityBinding::Global) {
        return Err(AppError::InvalidParameter(
            "This role pack uses global identity_binding; set identity globally instead of per scene."
                .to_string(),
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
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn set_user_relation(
    req: SetUserRelationRequest,
    state: State<'_, AppState>,
) -> Result<RoleInfo, String> {
    set_user_relation_impl(&state, &req).await
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn set_evolution_factor(
    req: SetEvolutionFactorRequest,
    state: State<'_, AppState>,
) -> Result<RoleInfo, String> {
    set_evolution_factor_impl(&state, &req).await
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
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
        return Err(AppError::RoleRuntimeNotReady.to_frontend_error());
    }
    state
        .db_manager
        .set_remote_life_enabled(&req.role_id, req.enabled)
        .await
        .map_err(|e| e.to_frontend_error())?;
    get_role_info_impl(state, &req.role_id, None).await
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn set_remote_life_enabled(
    req: SetRemoteLifeEnabledRequest,
    state: State<'_, AppState>,
) -> Result<RoleInfo, String> {
    set_remote_life_enabled_impl(&state, &req).await
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
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
        return Err(AppError::RoleRuntimeNotReady.to_frontend_error());
    }
    state
        .db_manager
        .set_interaction_mode_for_role(&req.role_id, req.mode.trim())
        .await
        .map_err(|e| e.to_frontend_error())?;
    get_role_info_impl(state, &req.role_id, None).await
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn set_role_interaction_mode(
    req: SetRoleInteractionModeRequest,
    state: State<'_, AppState>,
) -> Result<RoleInfo, String> {
    set_role_interaction_mode_impl(&state, &req).await
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
pub async fn set_session_plugin_backend_impl(
    state: &AppState,
    req: &SetSessionPluginBackendRequest,
) -> Result<RoleInfo, String> {
    let module = req.module.trim().to_ascii_lowercase();
    if req.local_memory_provider_id.is_some() && module.as_str() != "memory" {
        return Err(AppError::InvalidParameter(
            "local_memory_provider_id only supports module=memory".to_string(),
        )
        .to_frontend_error());
    }
    let slot_key = default_slot_key_for_module(&module).ok_or_else(|| {
        AppError::InvalidParameter(format!(
            "session backend override: unknown module {}",
            req.module
        ))
        .to_frontend_error()
    })?;
    let role = state
        .load_role_cached(&req.role_id)
        .map_err(|e| e.to_frontend_error())?;
    if role.slot_registry.is_none() {
        return Err(AppError::InvalidParameter(
            "v2 slot_registry required; run `oclive pack migrate-to-blueprint` on the role pack"
                .into(),
        )
        .to_frontend_error());
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
) -> Result<RoleInfo, String> {
    let slot_key = req.slot_key.trim();
    if slot_key.is_empty() {
        return Err(
            AppError::InvalidParameter("slot_key must not be empty".into()).to_frontend_error(),
        );
    }
    state
        .load_role_cached(&req.role_id)
        .map_err(|e| e.to_frontend_error())?;
    let ns = session_namespace(&req.role_id, req.session_id.as_deref());
    state
        .db_manager
        .ensure_role_runtime(ns.as_str())
        .await
        .map_err(|e| e.to_frontend_error())?;

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
) -> Result<RoleInfo, String> {
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
) -> Result<RoleInfo, String> {
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
) -> Result<RoleInfo, String> {
    let role_id = req.role_id.trim();
    if role_id.is_empty() {
        return Err(
            AppError::InvalidParameter("role_id must not be empty".into()).to_frontend_error(),
        );
    }
    state
        .storage
        .save_blueprint_v2_slot_registry(role_id, &req.slot_registry)
        .map_err(|e| e.to_frontend_error())?;
    state.invalidate_role_cache(role_id);
    state.invalidate_personality_cache_for_role(role_id);
    load_role_impl(state, role_id, false).await?;
    get_role_info_impl(state, role_id, None).await
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn save_role_slot_registry(
    req: SaveRoleSlotRegistryRequest,
    state: State<'_, AppState>,
) -> Result<RoleInfo, String> {
    save_role_slot_registry_impl(&state, &req).await
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn set_session_plugin_backend(
    req: SetSessionPluginBackendRequest,
    state: State<'_, AppState>,
) -> Result<RoleInfo, String> {
    set_session_plugin_backend_impl(&state, &req).await
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn set_session_slot_override(
    req: SetSessionSlotOverrideRequest,
    state: State<'_, AppState>,
) -> Result<RoleInfo, String> {
    set_session_slot_override_impl(&state, &req).await
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn clear_session_slot_override(
    req: ClearSessionSlotOverrideRequest,
    state: State<'_, AppState>,
) -> Result<RoleInfo, String> {
    clear_session_slot_override_impl(&state, &req).await
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn clear_all_session_slot_overrides(
    req: ClearAllSessionSlotOverridesRequest,
    state: State<'_, AppState>,
) -> Result<RoleInfo, String> {
    clear_all_session_slot_overrides_impl(&state, &req).await
}

#[derive(Debug, serde::Deserialize)]
pub struct ApplyAuthorSuggestedBackendsRequest {
    pub role_id: String,
    #[serde(default)]
    pub session_id: Option<String>,
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
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
            "This role pack has no author.json suggested_plugin_backends.".into(),
        )
        .to_frontend_error());
    };
    let ns = session_namespace(role_id, req.session_id.as_deref());
    state
        .db_manager
        .ensure_role_runtime(ns.as_str())
        .await
        .map_err(|e| e.to_frontend_error())?;
    let role_cached = state
        .load_role_cached(role_id)
        .map_err(|e| e.to_frontend_error())?;
    let Some(reg) = role_cached.slot_registry.as_ref() else {
        return Err(AppError::InvalidParameter(
            "v2 slot_registry required to apply author suggested backends".into(),
        )
        .to_frontend_error());
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
    get_role_info_impl(&state, role_id, req.session_id.as_deref()).await
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
pub async fn get_plugin_resolution_debug_impl(
    state: &AppState,
    req: &GetPluginResolutionDebugRequest,
) -> Result<PluginResolutionDebugInfo, String> {
    build_plugin_resolution_debug_info(state, &req.role_id, req.session_id.as_deref()).await
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
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
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn set_scene_user_relation(
    req: SetSceneUserRelationRequest,
    state: State<'_, AppState>,
) -> Result<RoleInfo, String> {
    set_scene_user_relation_impl(&state, &req).await
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn clear_scene_user_relation(
    req: ClearSceneUserRelationRequest,
    state: State<'_, AppState>,
) -> Result<RoleInfo, String> {
    clear_scene_user_relation_impl(&state, &req).await
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
/// 删除本地角色目录及该 manifest 角色（含 `__sess__` 会话命名空间）的 DB 状态。
pub async fn delete_role_impl(state: &AppState, role_id: String) -> Result<Value, String> {
    let rid = role_id.trim();
    if rid.is_empty() {
        return Err(AppError::InvalidParameter("role_id required".into()).to_frontend_error());
    }
    let removed_ns = state
        .db_manager
        .delete_all_data_for_manifest_role(rid)
        .await
        .map_err(|e| e.to_frontend_error())?;
    for ns in &removed_ns {
        state.clear_all_session_slot_overrides(ns);
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
#[must_use]
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

//! 角色 API：清单加载、运行时快照、身份与进化系数等 Tauri 命令。

mod display;
mod interaction;
mod runtime;

use crate::error::AppError;
use crate::models::dto::{
    ClearSceneUserRelationRequest, RoleData, RoleInfo, RoleSummary, SceneLabelEntry,
    SetEvolutionFactorRequest, SetRemoteLifeEnabledRequest, SetRoleInteractionModeRequest,
    SetSceneUserRelationRequest, SetUserRelationRequest, OCLIVE_DEFAULT_RELATION_SENTINEL,
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

const EVENT_IMPACT_MIN: f64 = 0.05;
const EVENT_IMPACT_MAX: f64 = 5.0;

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
    })
}

pub async fn get_role_info_impl(state: &AppState, role_id: &str) -> Result<RoleInfo, String> {
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

    let role = state
        .load_role_cached(role_id)
        .map_err(|e| e.to_frontend_error())?;

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
        knowledge_enabled,
        knowledge_chunk_count,
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
    get_role_info_impl(state, role_id).await
}

#[tauri::command]
pub async fn load_role(role_id: String, state: State<'_, AppState>) -> Result<RoleData, String> {
    load_role_impl(&state, &role_id, true).await
}

#[tauri::command]
pub async fn get_role_info(
    role_id: String,
    state: State<'_, AppState>,
) -> Result<RoleInfo, String> {
    get_role_info_impl(&state, &role_id).await
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
        return get_role_info_impl(state, &req.role_id).await;
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
    get_role_info_impl(state, &req.role_id).await
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
    get_role_info_impl(state, &req.role_id).await
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
    get_role_info_impl(state, &req.role_id).await
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
    get_role_info_impl(state, &req.role_id).await
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
    get_role_info_impl(state, &req.role_id).await
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
    get_role_info_impl(state, &req.role_id).await
}

#[tauri::command]
pub async fn set_role_interaction_mode(
    req: SetRoleInteractionModeRequest,
    state: State<'_, AppState>,
) -> Result<RoleInfo, String> {
    set_role_interaction_mode_impl(&state, &req).await
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

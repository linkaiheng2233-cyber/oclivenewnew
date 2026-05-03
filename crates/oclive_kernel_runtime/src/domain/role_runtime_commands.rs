//! 角色运行时设置：身份、场景身份、进化系数、异地心声、交互模式。
//!
//! 无 Tauri / 桌面依赖，供 `invoke`、OOCP、HTTP 或其它宿主复用。

use crate::domain::role_info_snapshot::get_role_info_snapshot;
use crate::env_flags;
use crate::error::{AppError, Result};
use crate::models::dto::{
    ClearSceneUserRelationRequest, OCLIVE_DEFAULT_RELATION_SENTINEL, RoleInfo, RoleSummary,
    SetEvolutionFactorRequest, SetRemoteLifeEnabledRequest, SetRoleInteractionModeRequest,
    SetSceneUserRelationRequest, SetUserRelationRequest,
};
use crate::models::role::IdentityBinding;
use crate::state::KernelAppState;

/// 与 `api/role` 历史常量一致；校验 `event_impact_factor` 写入。
pub const EVENT_IMPACT_FACTOR_MIN: f64 = 0.05;
pub const EVENT_IMPACT_FACTOR_MAX: f64 = 5.0;

pub async fn set_user_relation(
    state: &KernelAppState,
    req: &SetUserRelationRequest,
) -> Result<RoleInfo> {
    if !state.db_manager.role_runtime_exists(&req.role_id).await? {
        return Err(AppError::InvalidParameter(
            "Role runtime not initialized; call load_role first".to_string(),
        ));
    }
    let role = state.load_role_cached(&req.role_id)?;

    if matches!(role.identity_binding, IdentityBinding::Global) {
        state
            .db_manager
            .clear_all_scene_identities_for_role(&req.role_id)
            .await?;
    }

    if req.relation == OCLIVE_DEFAULT_RELATION_SENTINEL {
        state
            .db_manager
            .set_use_manifest_default(&req.role_id, true)
            .await?;
        let eff = role.default_relation.clone();
        let seed = role.initial_favorability_for_relation(eff.as_str());
        state
            .db_manager
            .ensure_identity_stats_row(&req.role_id, &eff, seed)
            .await?;
        state
            .db_manager
            .mirror_runtime_from_identity(&req.role_id, &eff)
            .await?;
        return get_role_info_snapshot(state, &req.role_id, None).await;
    }

    if !role.user_relations.iter().any(|r| r.id == req.relation) {
        return Err(AppError::InvalidParameter(format!(
            "unknown relation: {}",
            req.relation
        )));
    }
    state
        .db_manager
        .set_use_manifest_default(&req.role_id, false)
        .await?;
    state
        .db_manager
        .set_user_relation(&req.role_id, &req.relation)
        .await?;
    let seed = role.initial_favorability_for_relation(&req.relation);
    state
        .db_manager
        .ensure_identity_stats_row(&req.role_id, &req.relation, seed)
        .await?;
    state
        .db_manager
        .mirror_runtime_from_identity(&req.role_id, &req.relation)
        .await?;
    get_role_info_snapshot(state, &req.role_id, None).await
}

pub async fn clear_scene_user_relation(
    state: &KernelAppState,
    req: &ClearSceneUserRelationRequest,
) -> Result<RoleInfo> {
    if !state.db_manager.role_runtime_exists(&req.role_id).await? {
        return Err(AppError::InvalidParameter(
            "Role runtime not initialized; call load_role first".to_string(),
        ));
    }
    let role = state.load_role_cached(&req.role_id)?;
    if matches!(role.identity_binding, IdentityBinding::Global) {
        return Err(AppError::InvalidParameter(
            "当前角色包为全局身份模式（identity_binding: global），无需按场景清除身份覆盖"
                .to_string(),
        ));
    }
    let scenes = state.storage.list_scene_ids(&req.role_id)?;
    if !scenes.iter().any(|s| s == &req.scene_id) {
        return Err(AppError::InvalidParameter(format!(
            "scene_id not in role pack: {}",
            req.scene_id
        )));
    }
    state
        .db_manager
        .clear_user_relation_for_scene(&req.role_id, &req.scene_id)
        .await?;
    get_role_info_snapshot(state, &req.role_id, None).await
}

pub async fn set_scene_user_relation(
    state: &KernelAppState,
    req: &SetSceneUserRelationRequest,
) -> Result<RoleInfo> {
    if !state.db_manager.role_runtime_exists(&req.role_id).await? {
        return Err(AppError::InvalidParameter(
            "Role runtime not initialized; call load_role first".to_string(),
        ));
    }
    let role = state.load_role_cached(&req.role_id)?;
    if matches!(role.identity_binding, IdentityBinding::Global) {
        return Err(AppError::InvalidParameter(
            "当前角色包为全局身份模式（identity_binding: global），请使用全局身份设置，勿按场景绑定"
                .to_string(),
        ));
    }
    if !role.user_relations.iter().any(|r| r.id == req.relation) {
        return Err(AppError::InvalidParameter(format!(
            "unknown relation: {}",
            req.relation
        )));
    }
    let scenes = state.storage.list_scene_ids(&req.role_id)?;
    if !scenes.iter().any(|s| s == &req.scene_id) {
        return Err(AppError::InvalidParameter(format!(
            "scene_id not in role pack: {}",
            req.scene_id
        )));
    }
    state
        .db_manager
        .set_use_manifest_default(&req.role_id, false)
        .await?;
    state
        .db_manager
        .set_user_relation_for_scene(&req.role_id, &req.scene_id, &req.relation)
        .await?;
    get_role_info_snapshot(state, &req.role_id, None).await
}

pub async fn set_evolution_factor(
    state: &KernelAppState,
    req: &SetEvolutionFactorRequest,
) -> Result<RoleInfo> {
    let f = req.event_impact_factor;
    if !f.is_finite() || !(EVENT_IMPACT_FACTOR_MIN..=EVENT_IMPACT_FACTOR_MAX).contains(&f) {
        return Err(AppError::InvalidParameter(format!(
            "event_impact_factor must be between {} and {}",
            EVENT_IMPACT_FACTOR_MIN, EVENT_IMPACT_FACTOR_MAX
        )));
    }
    state.load_role_cached(&req.role_id)?;
    if !state.db_manager.role_runtime_exists(&req.role_id).await? {
        return Err(AppError::InvalidParameter(
            "Role runtime not initialized; call load_role first".to_string(),
        ));
    }
    state
        .db_manager
        .set_event_impact_factor(&req.role_id, f)
        .await?;
    get_role_info_snapshot(state, &req.role_id, None).await
}

pub async fn set_remote_life_enabled(
    state: &KernelAppState,
    req: &SetRemoteLifeEnabledRequest,
) -> Result<RoleInfo> {
    state.load_role_cached(&req.role_id)?;
    if !state.db_manager.role_runtime_exists(&req.role_id).await? {
        return Err(AppError::InvalidParameter(
            "Role runtime not initialized; call load_role first".to_string(),
        ));
    }
    state
        .db_manager
        .set_remote_life_enabled(&req.role_id, req.enabled)
        .await?;
    get_role_info_snapshot(state, &req.role_id, None).await
}

pub async fn set_role_interaction_mode(
    state: &KernelAppState,
    req: &SetRoleInteractionModeRequest,
) -> Result<RoleInfo> {
    state.load_role_cached(&req.role_id)?;
    if !state.db_manager.role_runtime_exists(&req.role_id).await? {
        return Err(AppError::InvalidParameter(
            "Role runtime not initialized; call load_role first".to_string(),
        ));
    }
    state
        .db_manager
        .set_interaction_mode_for_role(&req.role_id, req.mode.trim())
        .await?;
    get_role_info_snapshot(state, &req.role_id, None).await
}

/// 角色清单（manifest 简表）；`OCLIVE_LIST_DEV_ROLES` 控制是否包含 `dev_only` 包。
pub fn list_role_summaries(state: &KernelAppState) -> Result<Vec<RoleSummary>> {
    let list_dev = env_flags::list_dev_roles_enabled();
    let roles = state.storage.load_all_role_manifest_lite()?;
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

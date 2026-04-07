//! 运行时身份解析：`load_role` / `get_role_info` / 对话引擎共用同一套规则。

use crate::domain::user_identity::resolve_effective_user_relation_key;
use crate::models::dto::UserRelationDto;
use crate::models::role::Role;
use crate::state::AppState;

use super::display::user_relations_to_dto;

/// `load_role` / `get_role_info` 共用的运行时字段，避免两处漂移。
pub(crate) struct RoleRuntimeExtras {
    pub user_relations: Vec<UserRelationDto>,
    pub default_relation: String,
    pub current_user_relation: String,
    pub use_manifest_default: bool,
    pub event_impact_factor: f64,
}

async fn effective_event_impact(
    state: &AppState,
    role_id: &str,
    role: &Role,
) -> Result<f64, String> {
    Ok(state
        .db_manager
        .get_event_impact_factor(role_id)
        .await
        .map_err(|e| e.to_frontend_error())?
        .unwrap_or(role.evolution_config.event_impact_factor))
}

async fn effective_user_relation(
    state: &AppState,
    role_id: &str,
    scene_id: Option<&str>,
    role: &Role,
) -> Result<String, String> {
    resolve_effective_user_relation_key(state, role, role_id, scene_id)
        .await
        .map_err(|e| e.to_frontend_error())
}

pub(crate) async fn role_runtime_extras(
    state: &AppState,
    role_id: &str,
    scene_id: Option<&str>,
    role: &Role,
) -> Result<RoleRuntimeExtras, String> {
    let use_manifest_default = state
        .db_manager
        .get_use_manifest_default(role_id)
        .await
        .map_err(|e| e.to_frontend_error())?;
    Ok(RoleRuntimeExtras {
        user_relations: user_relations_to_dto(role),
        default_relation: role.default_relation.clone(),
        current_user_relation: effective_user_relation(state, role_id, scene_id, role).await?,
        use_manifest_default,
        event_impact_factor: effective_event_impact(state, role_id, role).await?,
    })
}

/// 尚无对话记忆且好感为 0 时，用当前身份对应的初始好感度写入 DB（仅一次）。
/// 调用方须先解析 `role_runtime_extras`（与 `current_favorability` 同源），避免同一次请求内重复查场景与身份。
pub(crate) async fn maybe_seed_initial_favorability_with_extras(
    state: &AppState,
    role_id: &str,
    role: &Role,
    rt: &RoleRuntimeExtras,
) -> Result<(), String> {
    let memory_count = state
        .memory_repo
        .count_memories(role_id)
        .await
        .map_err(|e| e.to_frontend_error())?;
    let eff = rt.current_user_relation.as_str();
    let seed = role.initial_favorability_for_relation(eff);
    state
        .db_manager
        .ensure_identity_stats_row(role_id, eff, seed)
        .await
        .map_err(|e| e.to_frontend_error())?;
    let fav = state
        .db_manager
        .get_favorability_for_identity(role_id, eff)
        .await
        .map_err(|e| e.to_frontend_error())?
        .unwrap_or(0.0);
    if memory_count > 0 || fav != 0.0 {
        return Ok(());
    }
    state
        .db_manager
        .set_identity_favorability_value(role_id, eff, seed)
        .await
        .map_err(|e| e.to_frontend_error())?;
    Ok(())
}

/// 与对话引擎一致：`role_identity_stats` 按有效身份键，缺失则回退全局 `role_runtime.favorability`。
pub(crate) async fn current_favorability_for_effective_identity(
    state: &AppState,
    role_id: &str,
    effective_relation_key: &str,
) -> Result<f64, String> {
    state
        .db_manager
        .favorability_for_identity_with_runtime_fallback(role_id, effective_relation_key)
        .await
        .map_err(|e| e.to_frontend_error())
}

/// 优先按身份键读 `role_identity_stats`，否则回退到全局 `role_runtime`（兼容旧数据）。
pub(crate) async fn resolve_relation_state_for_ui(
    state: &AppState,
    role_id: &str,
    effective_relation_key: &str,
) -> Result<String, String> {
    let mut relation_state = state
        .db_manager
        .get_relation_state_for_identity(role_id, effective_relation_key)
        .await
        .map_err(|e| e.to_frontend_error())?;
    if relation_state.is_none() {
        relation_state = state
            .db_manager
            .get_relation_state(role_id)
            .await
            .map_err(|e| e.to_frontend_error())?;
    }
    Ok(relation_state.unwrap_or_else(|| "Stranger".to_string()))
}

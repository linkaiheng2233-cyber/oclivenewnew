//! 角色 API：清单加载、运行时快照、身份与进化系数等 Tauri 命令。

use crate::models::dto::{
    ClearSceneUserRelationRequest, GetPluginResolutionDebugRequest, GetRoleInfoRequest,
    PluginResolutionDebugInfo, RoleData, RoleInfo, RoleSummary,
    SetEvolutionFactorRequest, SetRemoteLifeEnabledRequest, SetRoleInteractionModeRequest,
    SetSceneUserRelationRequest, SetSessionPluginBackendRequest, SetUserRelationRequest,
};
use crate::models::plugin_backends::PluginBackendsOverride;
use crate::state::AppState;
use tauri::State;

use serde::Deserialize;
use serde_json::Value;

/// `reset_portrait_emotion`：为 `true` 时（应用启动 `load_role`）立绘重置为 `neutral`；切换角色时为 `false` 以保留各角色上次立绘状态。
pub async fn load_role_impl(
    state: &AppState,
    role_id: &str,
    reset_portrait_emotion: bool,
) -> Result<RoleData, String> {
    oclive_kernel_runtime::domain::role_lifecycle::load_role(state, role_id, reset_portrait_emotion)
        .await
        .map_err(|e| e.to_frontend_error())
}

pub async fn get_role_info_impl(
    state: &AppState,
    role_id: &str,
    session_id: Option<&str>,
) -> Result<RoleInfo, String> {
    oclive_kernel_runtime::domain::role_info_snapshot::get_role_info_snapshot(
        state, role_id, session_id,
    )
    .await
    .map_err(|e| e.to_frontend_error())
}

pub async fn list_roles_impl(state: &AppState) -> Result<Vec<RoleSummary>, String> {
    oclive_kernel_runtime::domain::role_runtime_commands::list_role_summaries(state)
        .map_err(|e| e.to_frontend_error())
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
    oclive_kernel_runtime::domain::role_runtime_commands::set_user_relation(state, req)
        .await
        .map_err(|e| e.to_frontend_error())
}

pub async fn set_evolution_factor_impl(
    state: &AppState,
    req: &SetEvolutionFactorRequest,
) -> Result<RoleInfo, String> {
    oclive_kernel_runtime::domain::role_runtime_commands::set_evolution_factor(state, req)
        .await
        .map_err(|e| e.to_frontend_error())
}

pub async fn clear_scene_user_relation_impl(
    state: &AppState,
    req: &ClearSceneUserRelationRequest,
) -> Result<RoleInfo, String> {
    oclive_kernel_runtime::domain::role_runtime_commands::clear_scene_user_relation(state, req)
        .await
        .map_err(|e| e.to_frontend_error())
}

pub async fn set_scene_user_relation_impl(
    state: &AppState,
    req: &SetSceneUserRelationRequest,
) -> Result<RoleInfo, String> {
    oclive_kernel_runtime::domain::role_runtime_commands::set_scene_user_relation(state, req)
        .await
        .map_err(|e| e.to_frontend_error())
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
    oclive_kernel_runtime::domain::role_runtime_commands::set_remote_life_enabled(state, req)
        .await
        .map_err(|e| e.to_frontend_error())
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
    oclive_kernel_runtime::domain::role_runtime_commands::set_role_interaction_mode(state, req)
        .await
        .map_err(|e| e.to_frontend_error())
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
    oclive_kernel_runtime::domain::session_plugin_override::set_session_plugin_backend(state, req)
        .await
        .map_err(|e| e.to_frontend_error())
}

#[tauri::command]
pub async fn set_session_plugin_backend(
    req: SetSessionPluginBackendRequest,
    state: State<'_, AppState>,
) -> Result<RoleInfo, String> {
    set_session_plugin_backend_impl(&state, &req).await
}

#[derive(Debug, Clone, Deserialize)]
pub struct SetSessionPluginBackendsOverrideRequest {
    pub role_id: String,
    /// 可选：HTTP 试聊等多会话场景下指定会话 id；缺省表示角色默认会话。
    #[serde(default)]
    pub session_id: Option<String>,
    /// 会话级覆盖：仅 `Some` 字段会替换角色包 `plugin_backends` 对应模块；`directory_plugins` 亦可在此整体写入。
    #[serde(default)]
    pub override_backends: PluginBackendsOverride,
}

#[tauri::command]
pub async fn set_session_plugin_backends_override(
    req: SetSessionPluginBackendsOverrideRequest,
    state: State<'_, AppState>,
) -> Result<RoleInfo, String> {
    oclive_kernel_runtime::domain::session_plugin_override::set_session_plugin_backends_override(
        &state,
        req.role_id.as_str(),
        req.session_id.as_deref(),
        req.override_backends.clone(),
    )
    .await
    .map_err(|e| e.to_frontend_error())
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
    oclive_kernel_runtime::domain::session_plugin_override::apply_author_suggested_plugin_backends(
        &state,
        req.role_id.as_str(),
        req.session_id.as_deref(),
    )
    .await
    .map_err(|e| e.to_frontend_error())
}

pub async fn get_plugin_resolution_debug_impl(
    state: &AppState,
    req: &GetPluginResolutionDebugRequest,
) -> Result<PluginResolutionDebugInfo, String> {
    oclive_kernel_runtime::domain::plugin_resolution_debug::build_plugin_resolution_debug_info(
        state,
        &req.role_id,
        req.session_id.as_deref(),
        env!("CARGO_PKG_VERSION"),
    )
    .await
    .map_err(|e| e.to_frontend_error())
}

#[tauri::command]
pub async fn get_plugin_resolution_debug(
    req: GetPluginResolutionDebugRequest,
    state: State<'_, AppState>,
) -> Result<PluginResolutionDebugInfo, String> {
    get_plugin_resolution_debug_impl(&state, &req).await
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
    oclive_kernel_runtime::domain::role_lifecycle::delete_role(state, role_id)
        .await
        .map_err(|e| e.to_frontend_error())
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

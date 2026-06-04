//! Role API: manifest loading, runtime snapshots, identity, evolution factors, and related Tauri commands.
//!
//! Blueprint v2 disk writes go through [`save_role_slot_registry`]; there is **no** legacy Tauri command that only writes
//! `manifest.json`/`settings.json` `plugin_backends` (old packs are read-only via [`RoleStorage::load_role_from_legacy_manifest_dir`]).

pub mod expert;
pub mod evolution;
pub mod relation;
pub mod slot_session;

pub use oclive_kernel_host::service::role::{display, interaction, runtime};

pub use evolution::{
    set_evolution_factor_impl, set_remote_life_enabled_impl, set_role_interaction_mode_impl,
};
pub use relation::{
    clear_scene_user_relation_impl, set_scene_user_relation_impl, set_user_relation_impl,
};
pub use expert::{get_expert_routing, list_blueprint_includes, save_expert_routing};
pub use slot_session::{
    clear_all_session_slot_overrides_impl, clear_session_slot_override_impl,
    get_plugin_resolution_debug_impl, save_role_slot_registry_impl,
    set_session_plugin_backend_impl, set_session_slot_override_impl,
};
pub(crate) use slot_session::build_plugin_resolution_debug_info;

use crate::error::AppError;
use crate::api::error::CommandError;
use crate::models::dto::{GetRoleInfoRequest, RoleData, RoleInfo, RoleSummary};
use crate::state::{AppState, SharedAppState};
use tauri::{AppHandle, Manager, State};

use serde_json::{json, Value};

pub(crate) use crate::domain::role_snapshot::plugin_backends_override_from_slot_session;

pub use oclive_kernel_host::service::role::{
    ensure_manifest_role_ready, get_role_info_impl, list_roles_impl, load_role_impl,
    session_namespace, switch_role_impl,
};

pub(crate) const EVENT_IMPACT_MIN: f64 = 0.05;
pub(crate) const EVENT_IMPACT_MAX: f64 = 5.0;
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn load_role(
    role_id: String,
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<RoleData, CommandError> {
    if let Some(conn) = app.try_state::<crate::kernel_lifecycle::SharedKernelConnection>() {
        crate::kernel_attach::KernelHttpClient::load_role_via_http(&conn, role_id.trim())
            .await?;
    }
    load_role_impl(&state, &role_id, true).await
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn get_role_info(
    req: GetRoleInfoRequest,
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<RoleInfo, CommandError> {
    if let Some(conn) = app.try_state::<crate::kernel_lifecycle::SharedKernelConnection>() {
        match crate::kernel_attach::KernelHttpClient::get_role_info_via_http(&conn, &req).await {
            Ok(info) => return Ok(info),
            Err(AppError::RoleRuntimeNotReady) => {
                crate::kernel_attach::KernelHttpClient::load_role_via_http(
                    &conn,
                    req.role_id.trim(),
                )
                .await?;
                return crate::kernel_attach::KernelHttpClient::get_role_info_via_http(&conn, &req)
                    .await
                    .map_err(Into::into);
            }
            Err(e) => return Err(e.into()),
        }
    }
    get_role_info_impl(&state, &req.role_id, req.session_id.as_deref()).await
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn list_roles(state: State<'_, SharedAppState>) -> Result<Vec<RoleSummary>, CommandError> {
    list_roles_impl(&state).await
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn switch_role(
    role_id: String,
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<RoleInfo, CommandError> {
    if let Some(conn) = app.try_state::<crate::kernel_lifecycle::SharedKernelConnection>() {
        crate::kernel_attach::KernelHttpClient::load_role_via_http(&conn, role_id.trim())
            .await?;
        let req = GetRoleInfoRequest {
            role_id: role_id.clone(),
            session_id: None,
        };
        return crate::kernel_attach::KernelHttpClient::get_role_info_via_http(&conn, &req)
            .await
            .map_err(Into::into);
    }
    switch_role_impl(&state, &role_id).await
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
/// Deletes the local role directory and DB state for that manifest role (including `__sess__` session namespaces).
pub async fn delete_role_impl(state: &AppState, role_id: String) -> Result<Value, CommandError> {
    let rid = role_id.trim();
    if rid.is_empty() {
        return Err(AppError::InvalidParameter("role_id required".into()).into());
    }
    let removed_ns = state
        .db_manager
        .delete_all_data_for_manifest_role(rid)
        .await
        ?;
    let chat_location = state
        .load_role_cached_async(rid)
        .await
        .ok()
        .map(|r| r.pack_chat_storage_config.location.clone());
    let mirror_root = crate::infrastructure::chat_storage::resolve_role_chat_storage_root(
        state.directory_plugins.app_data_dir(),
        state.storage.roles_dir(),
        rid,
        chat_location.as_deref(),
    );
    if let Err(e) =
        crate::infrastructure::chat_storage::delete_mirror_tree_for_role(&mirror_root, rid).await
    {
        tracing::warn!(
            target: "oclive_chat_storage",
            role_id = %rid,
            error = %e,
            "delete_mirror_tree_for_role failed"
        );
    }
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

/// Strips the Windows verbatim path prefix `\\?\` to avoid frontend path issues.
fn path_string_for_frontend(p: &std::path::Path) -> String {
    let s = p.to_string_lossy();
    const VERBATIM: &str = "\\\\?\\";
    if let Some(stripped) = s.strip_prefix(VERBATIM) {
        stripped.to_string()
    } else {
        s.into_owned()
    }
}

/// Reads role pack asset bytes (`roles/{role_id}/{relative}`); returns `None` when the file does not exist.
///
/// # Errors
///
/// Returns a string when the file exists but cannot be read from disk.
#[tauri::command]
pub fn read_role_asset_bytes(
    role_id: String,
    relative: String,
    state: State<'_, SharedAppState>,
) -> Result<Option<Vec<u8>>, CommandError> {
    let p = state.storage.role_asset_path(&role_id, &relative);
    if !p.is_file() {
        return Ok(None);
    }
    Ok(Some(std::fs::read(&p)?))
}

/// Resolves the absolute path for `roles/{role_id}/{relative}`; when the file exists, the frontend can load it via `convertFileSrc`.
#[tauri::command]
#[must_use]
pub fn resolve_role_asset_path(
    role_id: String,
    relative: String,
    state: State<'_, SharedAppState>,
) -> Option<String> {
    let p = state.storage.role_asset_path(&role_id, &relative);
    if p.is_file() {
        return Some(path_string_for_frontend(&p));
    }
    None
}
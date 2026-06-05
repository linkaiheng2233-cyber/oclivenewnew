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
pub use oclive_kernel_host::service::role::{
    apply_author_suggested_plugin_backends_impl, clear_all_session_slot_overrides_impl,
    clear_session_slot_override_impl, get_plugin_resolution_debug_impl, save_role_slot_registry_impl,
    set_session_plugin_backend_impl, set_session_slot_override_impl,
};

use crate::error::AppError;
use crate::api::error::CommandError;
use crate::models::dto::{GetRoleInfoRequest, RoleData, RoleInfo, RoleSummary};
use crate::state::SharedAppState;
use tauri::{AppHandle, Manager, State};

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

pub use oclive_kernel_host::service::delete_role_impl;

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
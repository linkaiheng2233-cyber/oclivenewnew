use crate::api::error::CommandError;
use crate::infrastructure::{export_role_pack, import_role_pack, peek_role_pack_manifest};
use crate::models::dto::RolePackPeekResponse;
use crate::state::SharedAppState;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::Manager;
use tauri::State;
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn export_role_pack_command(
    role_id: String,
    dest_path: String,
    state: State<'_, SharedAppState>,
) -> Result<(), CommandError> {
    let p = PathBuf::from(dest_path);
    export_role_pack(&state.storage, &role_id, &p).map_err(Into::into)
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn peek_role_pack_command(
    src_path: String,
    _state: State<'_, SharedAppState>,
) -> Result<RolePackPeekResponse, CommandError> {
    let p = PathBuf::from(src_path);
    let (id, name, version) = tokio::task::spawn_blocking(move || peek_role_pack_manifest(&p))
        .await
        .map_err(|e| format!("预览任务异常: {}", e))??;
    Ok(RolePackPeekResponse { id, name, version })
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn import_role_pack_command(
    app: tauri::AppHandle,
    src_path: String,
    overwrite: bool,
    state: State<'_, SharedAppState>,
) -> Result<String, CommandError> {
    let storage = state.storage.clone();
    let path = PathBuf::from(src_path);
    let app = app.clone();
    let role_id = tokio::task::spawn_blocking(move || {
        import_role_pack(&storage, &path, overwrite, |prog| {
            let _ = app.emit_all("import_progress", prog);
        })
    })
    .await
    .map_err(|e| format!("导入任务异常: {}", e))??;

    let role = state.storage.load_role(&role_id)?;
    state.invalidate_personality_cache_for_role(&role_id);

    state
        .role_cache
        .write()
        .insert(role_id.clone(), Arc::new(role));

    Ok(role_id)
}

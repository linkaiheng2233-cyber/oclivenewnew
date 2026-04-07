use crate::error::AppError;
use crate::infrastructure::{export_role_pack, import_role_pack, peek_role_pack_manifest};
use crate::models::dto::RolePackPeekResponse;
use crate::state::AppState;
use std::path::PathBuf;
use tauri::Manager;
use tauri::State;

#[tauri::command]
pub async fn export_role_pack_command(
    role_id: String,
    dest_path: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let p = PathBuf::from(dest_path);
    export_role_pack(&state.storage, &role_id, &p).map_err(|e: AppError| e.to_frontend_error())
}

#[tauri::command]
pub async fn peek_role_pack_command(
    src_path: String,
    _state: State<'_, AppState>,
) -> Result<RolePackPeekResponse, String> {
    let p = PathBuf::from(src_path);
    let (id, name, version) =
        peek_role_pack_manifest(&p).map_err(|e: AppError| e.to_frontend_error())?;
    Ok(RolePackPeekResponse { id, name, version })
}

#[tauri::command]
pub async fn import_role_pack_command(
    app: tauri::AppHandle,
    src_path: String,
    overwrite: bool,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let storage = state.storage.clone();
    let path = PathBuf::from(src_path);
    let app = app.clone();
    let role_id = tokio::task::spawn_blocking(move || {
        import_role_pack(&storage, &path, overwrite, |prog| {
            let _ = app.emit_all("import_progress", prog);
        })
    })
    .await
    .map_err(|e| format!("导入任务异常: {}", e))?
    .map_err(|e: AppError| e.to_frontend_error())?;

    let role = state
        .storage
        .load_role(&role_id)
        .map_err(|e| e.to_frontend_error())?;
    state.role_cache.write().insert(role_id.clone(), role);

    Ok(role_id)
}

//! 在系统文件管理器中打开路径（供沉浸模式编辑角色包内 settings.json 等）。

use crate::state::AppState;
use tauri::{AppHandle, Manager, State};

#[tauri::command]
pub fn reveal_role_pack_folder(
    app: AppHandle,
    state: State<AppState>,
    role_id: String,
) -> Result<(), String> {
    let dir = oclive_kernel_runtime::domain::role_paths::role_pack_root_dir(&state, &role_id)
        .map_err(|e| e.to_frontend_error())?;
    tauri::api::shell::open(&app.shell_scope(), dir.to_string_lossy().to_string(), None)
        .map_err(|e| e.to_string())
}

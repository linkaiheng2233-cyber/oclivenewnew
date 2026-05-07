//! 全局快捷键：注册/注销与事件派发（`hotkey-action`）。

use crate::infrastructure::hotkey_bindings::{
    validate_hotkey_bindings, HotkeyAction, HotkeyBindingsFile,
};
use crate::state::AppState;
use serde::Serialize;
use tauri::{AppHandle, GlobalShortcutManager, Manager, State};

/// 将用户配置里常见的 `Ctrl+…` 在 macOS 上注册为 `Command+…`（与前端 `Meta` 一致）。
fn normalize_accelerator_for_os(accelerator: &str) -> String {
    #[cfg(target_os = "macos")]
    {
        let mut s = accelerator.to_string();
        for (from, to) in [
            ("Ctrl+", "Command+"),
            ("CTRL+", "Command+"),
            ("ctrl+", "Command+"),
        ] {
            if s.contains(from) {
                s = s.replace(from, to);
            }
        }
        s
    }
    #[cfg(not(target_os = "macos"))]
    {
        accelerator.to_string()
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct HotkeyActionEvent {
    binding_id: String,
    action: HotkeyAction,
}

/// 注销全部后按配置注册；仅 `enabled` 为真且 `accelerator` 非空的条目会注册。
pub fn apply_global_hotkeys(app: &AppHandle, file: &HotkeyBindingsFile) -> Result<(), String> {
    validate_hotkey_bindings(file)?;
    let mut mgr = app.global_shortcut_manager();
    mgr.unregister_all().map_err(|e| e.to_string())?;
    for b in &file.bindings {
        if !b.enabled {
            continue;
        }
        let acc = b.accelerator.trim();
        if acc.is_empty() {
            continue;
        }
        let app_clone = app.clone();
        let id = b.id.clone();
        let action = b.action.clone();
        let acc_owned = normalize_accelerator_for_os(acc);
        mgr.register(&acc_owned, move || {
            let payload = HotkeyActionEvent {
                binding_id: id.clone(),
                action: action.clone(),
            };
            let _ = app_clone.emit_all("hotkey-action", payload);
        })
        .map_err(|e| format!("register {}: {}", acc_owned, e))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn get_hotkey_bindings(state: State<'_, AppState>) -> Result<HotkeyBindingsFile, String> {
    Ok(HotkeyBindingsFile::load_async(state.directory_plugins.app_data_dir()).await)
}

#[tauri::command]
pub async fn save_hotkey_bindings(
    bindings: HotkeyBindingsFile,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_hotkey_bindings(&bindings)?;
    bindings
        .save_async(state.directory_plugins.app_data_dir())
        .await?;
    apply_global_hotkeys(&app, &bindings)
}

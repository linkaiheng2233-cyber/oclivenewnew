//! 全局快捷键：注册/注销与事件派发（`hotkey-action`）。

use crate::infrastructure::hotkey_bindings::{HotkeyAction, HotkeyBindingsFile};
use crate::state::AppState;
use serde::Serialize;
use tauri::{AppHandle, GlobalShortcutManager, Manager, State};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct HotkeyActionEvent {
    binding_id: String,
    action: HotkeyAction,
}

fn validate_hotkey_bindings(file: &HotkeyBindingsFile) -> Result<(), String> {
    let mut seen = std::collections::HashSet::new();
    for b in &file.bindings {
        if !b.enabled {
            continue;
        }
        let acc = b.accelerator.trim();
        if acc.is_empty() {
            continue;
        }
        if !seen.insert(acc.to_string()) {
            return Err(format!("重复的已启用快捷键：{}", acc));
        }
    }
    Ok(())
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
        let acc_owned = acc.to_string();
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
pub fn get_hotkey_bindings(state: State<'_, AppState>) -> Result<HotkeyBindingsFile, String> {
    Ok(HotkeyBindingsFile::load(
        state.directory_plugins.app_data_dir(),
    ))
}

#[tauri::command]
pub fn save_hotkey_bindings(
    bindings: HotkeyBindingsFile,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validate_hotkey_bindings(&bindings)?;
    bindings.save(state.directory_plugins.app_data_dir())?;
    apply_global_hotkeys(&app, &bindings)
}

//! Global hotkeys: register/unregister and event dispatch (`hotkey-action`).

use crate::api::error::CommandError;
use oclive_kernel_host::infrastructure::hotkey_bindings::{HotkeyAction, HotkeyBindingsFile};
use oclive_kernel_host::state::{AppState, SharedAppState};
use serde::Serialize;
use tauri::{AppHandle, GlobalShortcutManager, Manager, State};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct HotkeyActionEvent {
    binding_id: String,
    action: HotkeyAction,
}

fn validate_hotkey_bindings(file: &HotkeyBindingsFile) -> Result<(), CommandError> {
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
            return Err(format!("重复的已启用快捷键：{}", acc).into());
        }
    }
    Ok(())
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
/// Unregister all, then register from config; only entries with `enabled` true and non-empty `accelerator` are registered.
pub fn apply_global_hotkeys(
    app: &AppHandle,
    file: &HotkeyBindingsFile,
) -> Result<(), CommandError> {
    validate_hotkey_bindings(file)?;
    let mut mgr = app.global_shortcut_manager();
    mgr.unregister_all()
        .map_err(|e| CommandError::from(e.to_string()))?;
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
        .map_err(|e| CommandError::from(format!("register {}: {}", acc_owned, e)))?;
    }
    Ok(())
}

/// Same logic as [`get_hotkey_bindings`]; for integration tests without `State` wrapper.
///
/// # Errors
///
/// Returns `Err(String)` when the hotkey config file cannot be read or parsed.
pub fn get_hotkey_bindings_impl(state: &AppState) -> Result<HotkeyBindingsFile, CommandError> {
    Ok(HotkeyBindingsFile::load(
        state.directory_plugins.app_data_dir(),
    ))
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub fn get_hotkey_bindings(
    state: State<'_, SharedAppState>,
) -> Result<HotkeyBindingsFile, CommandError> {
    get_hotkey_bindings_impl(&state)
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub fn save_hotkey_bindings(
    bindings: HotkeyBindingsFile,
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<(), CommandError> {
    validate_hotkey_bindings(&bindings)?;
    bindings.save(state.directory_plugins.app_data_dir())?;
    apply_global_hotkeys(&app, &bindings)
}

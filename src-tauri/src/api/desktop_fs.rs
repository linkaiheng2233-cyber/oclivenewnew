//! Desktop file helpers exposed via Tauri commands (replaces direct `@tauri-apps/api/fs` IPC).

use crate::api::error::CommandError;
use std::fs;
use std::path::Path;

/// Write UTF-8 text to a user-selected absolute path (e.g. export save dialog).
///
/// # Errors
///
/// Returns disk IO errors as strings when the path cannot be created or written.
#[tauri::command]
pub fn write_user_text_file(path: String, contents: String) -> Result<(), CommandError> {
    let p = Path::new(path.trim());
    if p.as_os_str().is_empty() {
        return Err("empty path".to_string().into());
    }
    if let Some(parent) = p.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    fs::write(p, contents)?;
    Ok(())
}

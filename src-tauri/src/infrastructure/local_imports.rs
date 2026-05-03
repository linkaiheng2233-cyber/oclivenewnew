//! 桌面适配：`app_data` → 导入根目录，业务逻辑见 `oclive_kernel_runtime::domain::local_imports`。

use crate::state::AppState;
use oclive_kernel_runtime::domain::local_imports as kernel;
use std::path::PathBuf;

pub use kernel::{LocalImportCandidate, LocalImportKind};

#[inline]
pub fn imports_root(state: &AppState) -> PathBuf {
    kernel::imports_root(state.directory_plugins.app_data_dir())
}

pub fn ensure_import_folders_exist(state: &AppState) -> Result<(), String> {
    let root = imports_root(state);
    kernel::ensure_import_folders_exist(&root).map_err(|e| e.to_frontend_error())
}

pub fn list_local_import_candidates(state: &AppState) -> Result<Vec<LocalImportCandidate>, String> {
    let root = imports_root(state);
    kernel::list_local_import_candidates(&root).map_err(|e| e.to_frontend_error())
}

pub fn read_import_text(path: &std::path::Path, max_bytes: usize) -> Result<String, String> {
    kernel::read_import_text(path, max_bytes).map_err(|e| e.to_frontend_error())
}

pub fn resolve_path_under_imports_root(
    user_path: &str,
    state: &AppState,
) -> Result<PathBuf, String> {
    let root = imports_root(state);
    kernel::resolve_path_under_imports_root(user_path, &root).map_err(|e| e.to_frontend_error())
}

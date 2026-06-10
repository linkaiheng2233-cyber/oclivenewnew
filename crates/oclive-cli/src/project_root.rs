//! Shared project-root resolution for bench / build / registry commands.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Canonicalize a project root from CLI `-o` / cwd-relative path.
///
/// # Errors
///
/// Fails when the path cannot be resolved or canonicalized.
pub fn resolve_project_root(path: &Path) -> Result<PathBuf> {
    let root = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().context("current_dir")?.join(path)
    };
    root.canonicalize()
        .with_context(|| format!("cannot resolve project path: {}", root.display()))
}

/// Resolve a registry push/pull root: explicit path or local registry entry.
///
/// # Errors
///
/// Fails when the path cannot be resolved or the registry has no matching entry.
pub fn resolve_project_root_for_registry(name: &str, path: Option<&Path>) -> Result<PathBuf> {
    if let Some(p) = path {
        return resolve_project_root(p);
    }
    let entry = crate::registry::find_entry(name)?.ok_or_else(|| {
        anyhow::anyhow!("No project {name} in local registry; use registry add or -o with a path")
    })?;
    PathBuf::from(&entry.path)
        .canonicalize()
        .with_context(|| format!("registry path {}", entry.path))
}

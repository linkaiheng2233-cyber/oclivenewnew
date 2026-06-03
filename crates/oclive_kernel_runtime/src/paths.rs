//! Canonical cross-host app data paths (`OCLive/data`) and `--api` resolution.

use std::path::{Path, PathBuf};

/// Explicit app data root (VS Code spawn, desktop override).
pub const ENV_APP_DATA: &str = "OCLIVE_APP_DATA";

/// When `1` or `true`, `--api` uses ephemeral temp dirs (CI / OOCP default).
pub const ENV_APP_DATA_LEGACY_TEMP: &str = "OCLIVE_API_USE_TEMP_APP_DATA";

/// When `1` or `true` (and no explicit temp flag), `--api` uses the brand canonical dir.
pub const ENV_USE_CANONICAL_APP_DATA: &str = "OCLIVE_USE_CANONICAL_APP_DATA";

/// Skip one-time Tauri → canonical copy (tests).
pub const ENV_SKIP_APP_DATA_MIGRATION: &str = "OCLIVE_SKIP_APP_DATA_MIGRATION";

/// Tauri bundle identifier — legacy app data folder name segment.
pub const TAURI_APP_IDENTIFIER: &str = "com.oclivenewnew.app";

/// How [`resolve_app_data_dir_for_api`] chose the directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppDataMode {
    /// `OCLIVE_APP_DATA` or canonical brand path; persisted across restarts.
    Persistent,
    /// Ephemeral under the system temp directory (deleted on drop when wired).
    Temp,
}

/// Returns true when `key` is `1` or `true` (case-insensitive).
#[must_use]
pub fn env_flag_is_truthy(key: &str) -> bool {
    env_is_truthy(key)
}

fn env_is_truthy(key: &str) -> bool {
    std::env::var(key)
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

fn env_non_empty_path(key: &str) -> Option<PathBuf> {
    let v = std::env::var(key).ok()?;
    let trimmed = v.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(PathBuf::from(trimmed))
    }
}

/// Brand data root: `%LOCALAPPDATA%/OCLive/data` (Windows), etc.
#[must_use]
pub fn canonical_brand_app_data_dir() -> PathBuf {
    canonical_brand_parent().join("data")
}

fn canonical_brand_parent() -> PathBuf {
    if let Some(base) = std::env::var_os("XDG_DATA_HOME")
        .filter(|_| cfg!(target_os = "linux"))
        .map(PathBuf::from)
    {
        return base.join("OCLive");
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(local) = std::env::var_os("LOCALAPPDATA").map(PathBuf::from) {
            return local.join("OCLive");
        }
        if let Some(home) = dirs_home() {
            return home.join("OCLive");
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = dirs_home() {
            return home
                .join("Library")
                .join("Application Support")
                .join("OCLive");
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(home) = dirs_home() {
            return home.join(".local").join("share").join("OCLive");
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        if let Some(home) = dirs_home() {
            return home.join(".oclive");
        }
    }

    std::env::temp_dir().join("OCLive")
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

/// Legacy Tauri desktop app data (pre-unification).
#[must_use]
pub fn tauri_legacy_app_data_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Some(appdata) = std::env::var_os("APPDATA").map(PathBuf::from) {
            return appdata.join(TAURI_APP_IDENTIFIER);
        }
        if let Some(home) = dirs_home() {
            return home.join(TAURI_APP_IDENTIFIER);
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = dirs_home() {
            return home
                .join("Library")
                .join("Application Support")
                .join(TAURI_APP_IDENTIFIER);
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(xdg) = std::env::var_os("XDG_DATA_HOME").map(PathBuf::from) {
            return xdg.join(TAURI_APP_IDENTIFIER);
        }
        if let Some(home) = dirs_home() {
            return home.join(".local").join("share").join(TAURI_APP_IDENTIFIER);
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        if let Some(home) = dirs_home() {
            return home.join(TAURI_APP_IDENTIFIER);
        }
    }

    std::env::temp_dir().join(TAURI_APP_IDENTIFIER)
}

/// `app_data/app.db`
#[must_use]
pub fn resolve_db_path(app_data: &Path) -> PathBuf {
    app_data.join("app.db")
}

/// Desktop / spawn default: canonical brand dir (honours `OCLIVE_APP_DATA` when set).
#[must_use]
pub fn resolve_app_data_dir_for_host() -> PathBuf {
    env_non_empty_path(ENV_APP_DATA).unwrap_or_else(canonical_brand_app_data_dir)
}

/// Temp `--api` layout for a listen port (CI / unset opt-in).
#[must_use]
pub fn temp_api_app_data_dir(port: u16) -> PathBuf {
    let db_path = std::env::temp_dir().join(format!("oclive_api_{port}.db"));
    db_path
        .parent()
        .map(|p| p.join("oclive_api_app_data"))
        .unwrap_or_else(|| std::env::temp_dir().join("oclive_api_app_data"))
}

#[must_use]
pub fn temp_api_db_path(port: u16) -> PathBuf {
    std::env::temp_dir().join(format!("oclive_api_{port}.db"))
}

/// Resolve app data for `serve_api` / headless kernel.
///
/// Priority: `OCLIVE_APP_DATA` → forced temp → canonical opt-in → temp (default).
#[must_use]
pub fn resolve_app_data_dir_for_api(port: u16) -> (PathBuf, AppDataMode) {
    if let Some(explicit) = env_non_empty_path(ENV_APP_DATA) {
        return (explicit, AppDataMode::Persistent);
    }
    if env_is_truthy(ENV_APP_DATA_LEGACY_TEMP) {
        return (temp_api_app_data_dir(port), AppDataMode::Temp);
    }
    if env_is_truthy(ENV_USE_CANONICAL_APP_DATA) {
        return (
            canonical_brand_app_data_dir(),
            AppDataMode::Persistent,
        );
    }
    (temp_api_app_data_dir(port), AppDataMode::Temp)
}

/// Create `app_data` if missing; returns the same path.
///
/// # Errors
///
/// Returns I/O error message when the directory cannot be created.
pub fn ensure_app_data_dir(path: &Path) -> Result<PathBuf, String> {
    std::fs::create_dir_all(path).map_err(|e| {
        format!(
            "create app_data_dir {}: {e}",
            path.display()
        )
    })?;
    Ok(path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_db_path_joins_app_db() {
        let p = resolve_db_path(Path::new("/data/OCLive"));
        assert_eq!(p, Path::new("/data/OCLive/app.db"));
    }

    #[test]
    fn canonical_brand_ends_with_oclive_data() {
        let p = canonical_brand_app_data_dir();
        assert!(p.ends_with("data") || p.to_string_lossy().contains("OCLive"));
    }
}

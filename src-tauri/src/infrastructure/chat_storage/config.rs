//! Chat storage root path resolution (`OCLIVE_CHAT_STORAGE_ROOT` > app setting > `{app_data}/chats/`).

use crate::error::{AppError, Result};
use parking_lot::RwLock;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

const DEFAULT_SUBDIR: &str = "chats";

/// Environment variable override for chat JSON mirror root (see `handoff/CHAT_STORAGE_ARCHITECTURE.md`).
pub const ENV_CHAT_STORAGE_ROOT: &str = "OCLIVE_CHAT_STORAGE_ROOT";

/// `app_settings` key for user-chosen mirror root (below env, above default).
pub const APP_SETTING_CHAT_STORAGE_ROOT: &str = "chat_storage_root";

static PERSISTED_STORAGE_ROOT: OnceLock<RwLock<Option<PathBuf>>> = OnceLock::new();

/// Process-wide persisted root (loaded at startup; updated via `set_chat_storage_root`).
#[must_use]
pub fn chat_storage_root_override() -> &'static RwLock<Option<PathBuf>> {
    PERSISTED_STORAGE_ROOT.get_or_init(|| RwLock::new(None))
}

pub fn set_persisted_storage_root(path: Option<PathBuf>) {
    *chat_storage_root_override().write() = path;
}

/// Global default max messages per session (user + assistant rows combined).
pub const DEFAULT_MAX_MESSAGES: i64 = 500;

/// Resolve per-role cap from pack config (falls back to [`DEFAULT_MAX_MESSAGES`]).
#[must_use]
pub fn resolve_max_messages_per_session(configured: Option<u32>) -> i64 {
    configured
        .map(|n| i64::from(n.max(2)))
        .unwrap_or(DEFAULT_MAX_MESSAGES)
}

/// Resolve storage root: env > persisted app setting > `{app_data_dir}/chats/`.
#[must_use]
pub fn resolve_storage_root(app_data_dir: &Path) -> PathBuf {
    if let Ok(raw) = std::env::var(ENV_CHAT_STORAGE_ROOT) {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    if let Some(p) = chat_storage_root_override().read().as_ref() {
        return p.clone();
    }
    app_data_dir.join(DEFAULT_SUBDIR)
}

/// Probes whether a directory is writable (checks permissions if it exists, parent otherwise).
/// On Windows, also performs a probe write (temp file) beyond `readonly()` for more reliable fallback.
#[must_use]
pub fn is_path_writable(path: &Path) -> bool {
    if path.exists() {
        match std::fs::metadata(path) {
            Ok(m) => {
                if m.permissions().readonly() {
                    return false;
                }
            }
            Err(_) => return false,
        }
    }
    let probe = path.join(".oclive_write_probe");
    match std::fs::write(&probe, b"1") {
        Ok(()) => {
            let _ = std::fs::remove_file(&probe);
            true
        }
        Err(_) => false,
    }
}

/// Resolves chat log root with priority:
/// 1. env `OCLIVE_CHAT_STORAGE_ROOT` (forced override, unchanged)
/// 2. role pack `config.json` → `chat_storage.location` = `"role_pack"` + writable role pack dir
///    warns and falls back when not writable
/// 3. global persisted setting `chat_storage_root` (unchanged)
/// 4. default `{app_data}/chats/` (unchanged)
#[must_use]
pub fn resolve_storage_root_with_role(
    app_data_dir: &Path,
    role_config: &crate::models::RolePackChatStorageConfig,
    role_pack_dir: Option<&Path>,
) -> PathBuf {
    if let Ok(raw) = std::env::var(ENV_CHAT_STORAGE_ROOT) {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    if role_config.location == "role_pack" {
        if let Some(role_dir) = role_pack_dir {
            let role_chats = role_dir.join(DEFAULT_SUBDIR);
            if is_path_writable(role_dir) {
                return role_chats;
            }
            tracing::warn!(
                role_dir = %role_dir.display(),
                "role_pack chat_storage.location 但角色包目录不可写，回退到全局路径"
            );
        } else {
            tracing::warn!(
                "chat_storage.location = 'role_pack' 但未提供角色包目录，回退到全局路径"
            );
        }
    }
    if let Some(p) = chat_storage_root_override().read().as_ref() {
        return p.clone();
    }
    app_data_dir.join(DEFAULT_SUBDIR)
}

/// Read `chat_storage.location` from `{roles_dir}/{role_id}/config.json`.
#[must_use]
pub fn read_role_chat_storage_location(roles_dir: &Path, role_id: &str) -> String {
    let path = roles_dir.join(role_id).join("config.json");
    std::fs::read_to_string(path).ok().and_then(|raw| {
        serde_json::from_str::<crate::models::RolePackConfigFile>(&raw)
            .ok()
            .map(|c| c.chat_storage.location)
    }).unwrap_or_else(|| "global".to_string())
}

/// Resolve mirror root for a manifest role (per-turn / per-role operations).
#[must_use]
pub fn resolve_role_chat_storage_root(
    app_data_dir: &Path,
    roles_dir: &Path,
    role_id: &str,
    location: Option<&str>,
) -> PathBuf {
    let loc = location
        .map(str::to_string)
        .unwrap_or_else(|| read_role_chat_storage_location(roles_dir, role_id));
    let cfg = crate::models::RolePackChatStorageConfig {
        location: loc,
        ..Default::default()
    };
    resolve_storage_root_with_role(
        app_data_dir,
        &cfg,
        Some(&roles_dir.join(role_id)),
    )
}

/// Copy mirror tree when changing storage root (best-effort).
///
/// # Errors
///
/// IO errors propagate; skips when `from` is missing; errors when `to` already exists.
pub async fn migrate_mirror_tree(from: &Path, to: &Path) -> Result<()> {
    if from == to {
        return Ok(());
    }
    if !from.is_dir() {
        return Ok(());
    }
    if to.exists() {
        return Err(AppError::InvalidParameter(format!(
            "target chat storage root already exists: {}",
            to.display()
        )));
    }
    copy_dir_recursive(from, to).await
}

async fn copy_dir_recursive(from: &Path, to: &Path) -> Result<()> {
    tokio::fs::create_dir_all(to).await.map_err(AppError::IoError)?;
    let mut entries = tokio::fs::read_dir(from).await.map_err(AppError::IoError)?;
    while let Some(entry) = entries.next_entry().await.map_err(AppError::IoError)? {
        let ft = entry.file_type().await.map_err(AppError::IoError)?;
        let dest = to.join(entry.file_name());
        if ft.is_dir() {
            Box::pin(copy_dir_recursive(&entry.path(), &dest)).await?;
        } else {
            tokio::fs::copy(entry.path(), dest)
                .await
                .map_err(AppError::IoError)?;
        }
    }
    Ok(())
}

/// `{root}/{role_id}/{scene_id}/` with sanitized path segments.
///
/// # Errors
///
/// Returns [`AppError::InvalidParameter`] when segments are empty after sanitization.
pub fn resolve_session_dir(root: &Path, role_id: &str, scene_id: &str) -> Result<PathBuf> {
    let role = sanitize_path_segment(role_id)?;
    let scene = sanitize_path_segment(scene_id)?;
    Ok(root.join(role).join(scene))
}

/// Safe single path segment (no separators or Windows-forbidden chars).
pub fn sanitize_path_segment(raw: &str) -> Result<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidParameter(
            "chat storage path segment empty".to_string(),
        ));
    }
    let mut out = String::new();
    for c in trimmed.chars() {
        if c == '/' || c == '\\' || c == ':' || c == '*' || c == '?' || c == '"' || c == '<' || c == '>' || c == '|' {
            out.push('_');
        } else if c.is_control() {
            continue;
        } else {
            out.push(c);
        }
    }
    if out.is_empty() || out == "." || out == ".." {
        return Err(AppError::InvalidParameter(format!(
            "invalid chat storage path segment: {raw:?}"
        )));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_root_overrides_default() {
        let tmp = std::env::temp_dir().join("oclive_chat_storage_test");
        std::env::set_var(ENV_CHAT_STORAGE_ROOT, tmp.to_string_lossy().as_ref());
        let root = resolve_storage_root(Path::new("/app_data"));
        assert_eq!(root, tmp);
        std::env::remove_var(ENV_CHAT_STORAGE_ROOT);
    }

    #[test]
    fn default_root_under_app_data() {
        std::env::remove_var(ENV_CHAT_STORAGE_ROOT);
        let root = resolve_storage_root(Path::new("/app_data"));
        assert_eq!(root, Path::new("/app_data/chats"));
    }

    #[test]
    fn sanitize_replaces_invalid_chars() {
        assert_eq!(sanitize_path_segment("mumu").unwrap(), "mumu");
        assert_eq!(sanitize_path_segment("a/b").unwrap(), "a_b");
    }

    #[test]
    fn role_pack_location_resolves_to_role_dir_chats() {
        let role_dir = std::env::temp_dir().join("oclive_chat_storage_role_test");
        let _ = std::fs::create_dir_all(&role_dir);
        let config = crate::models::RolePackChatStorageConfig {
            location: "role_pack".to_string(),
            ..Default::default()
        };
        let root = resolve_storage_root_with_role(
            Path::new("/app_data"),
            &config,
            Some(&role_dir),
        );
        assert_eq!(root, role_dir.join("chats"));
        let _ = std::fs::remove_dir_all(&role_dir);
    }

    #[test]
    fn global_location_uses_default_app_data() {
        let role_dir = std::env::temp_dir().join("oclive_chat_storage_global_test");
        let _ = std::fs::create_dir_all(&role_dir);
        let config = crate::models::RolePackChatStorageConfig {
            location: "global".to_string(),
            ..Default::default()
        };
        let root = resolve_storage_root_with_role(
            Path::new("/app_data"),
            &config,
            Some(&role_dir),
        );
        assert_eq!(root, Path::new("/app_data/chats"));
        let _ = std::fs::remove_dir_all(&role_dir);
    }

    #[test]
    fn missing_location_defaults_to_global() {
        let config = crate::models::RolePackChatStorageConfig::default();
        let root = resolve_storage_root_with_role(
            Path::new("/app_data"),
            &config,
            None,
        );
        assert_eq!(root, Path::new("/app_data/chats"));
    }

    #[test]
    fn role_pack_unwritable_falls_back_to_global() {
        let nonexistent =
            std::path::PathBuf::from("/nonexistent_readonly_oclive_test/role");
        let config = crate::models::RolePackChatStorageConfig {
            location: "role_pack".to_string(),
            ..Default::default()
        };
        let root = resolve_storage_root_with_role(
            Path::new("/app_data"),
            &config,
            Some(&nonexistent),
        );
        assert_eq!(root, Path::new("/app_data/chats"));
    }
}

//! Chat storage root path resolution (`OCLIVE_CHAT_STORAGE_ROOT` > `{app_data}/chats/`).

use crate::error::{AppError, Result};
use std::path::{Path, PathBuf};

const DEFAULT_SUBDIR: &str = "chats";

/// Environment variable override for chat JSON mirror root (see `handoff/CHAT_STORAGE_ARCHITECTURE.md`).
pub const ENV_CHAT_STORAGE_ROOT: &str = "OCLIVE_CHAT_STORAGE_ROOT";

/// Global default max messages per session (user + assistant rows combined).
pub const DEFAULT_MAX_MESSAGES: i64 = 500;

/// Alias kept for existing call sites.
pub const MAX_MESSAGES_PER_SESSION: i64 = DEFAULT_MAX_MESSAGES;

/// Resolve per-role cap from pack config (falls back to [`DEFAULT_MAX_MESSAGES`]).
#[must_use]
pub fn resolve_max_messages_per_session(configured: Option<u32>) -> i64 {
    configured
        .map(|n| i64::from(n.max(2)))
        .unwrap_or(DEFAULT_MAX_MESSAGES)
}

/// Resolve storage root: `OCLIVE_CHAT_STORAGE_ROOT` or `{app_data_dir}/chats/`.
#[must_use]
pub fn resolve_storage_root(app_data_dir: &Path) -> PathBuf {
    if let Ok(raw) = std::env::var(ENV_CHAT_STORAGE_ROOT) {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    app_data_dir.join(DEFAULT_SUBDIR)
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
}

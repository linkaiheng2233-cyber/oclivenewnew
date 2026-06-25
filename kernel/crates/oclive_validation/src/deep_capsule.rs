//! Wave D · `prompts/deep_capsule.txt` validation.

use std::fs;
use std::path::Path;

/// Maximum Han / visible character count for `prompts/deep_capsule.txt`.
pub const DEEP_CAPSULE_MAX_CHARS: usize = 2500;

/// Validates optional Deep persona capsule file for a role pack directory.
///
/// # Errors
///
/// Returns `Err(Vec<String>)` when `enabled` but file missing/empty, or when over length limit.
pub fn validate_deep_capsule_file(role_dir: &Path, enabled: bool) -> Result<(), Vec<String>> {
    let path = role_dir.join("prompts/deep_capsule.txt");
    let exists = path.is_file();
    if enabled && !exists {
        return Err(vec![format!(
            "meta.deep_capsule_enabled=true 但缺少 prompts/deep_capsule.txt（{}）",
            path.display()
        )]);
    }
    if !exists {
        return Ok(());
    }
    let content = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            return Err(vec![format!(
                "deep_capsule.txt 不可读: {} — {e}",
                path.display()
            )]);
        }
    };
    if content.trim().is_empty() {
        return Err(vec![format!("deep_capsule.txt 为空: {}", path.display())]);
    }
    let char_count = content.chars().count();
    if char_count > DEEP_CAPSULE_MAX_CHARS {
        return Err(vec![format!(
            "deep_capsule.txt 超过 {DEEP_CAPSULE_MAX_CHARS} 字（当前 {char_count} 字）"
        )]);
    }
    Ok(())
}

#[must_use]
pub fn blueprint_meta_deep_capsule_enabled(role_dir: &Path) -> bool {
    let blueprint_path = role_dir.join(crate::blueprint_v2::PIPELINE_BLUEPRINT_FILENAME);
    if !blueprint_path.is_file() {
        return false;
    }
    let Ok(raw) = fs::read_to_string(&blueprint_path) else {
        return false;
    };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return false;
    };
    v.get("meta")
        .and_then(|m| m.get("deep_capsule_enabled"))
        .and_then(|b| b.as_bool())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn rejects_enabled_without_file() {
        let dir = tempfile::tempdir().unwrap();
        let err = validate_deep_capsule_file(dir.path(), true).unwrap_err();
        assert!(err[0].contains("deep_capsule_enabled"));
    }

    #[test]
    fn rejects_over_limit() {
        let dir = tempfile::tempdir().unwrap();
        let prompts = dir.path().join("prompts");
        fs::create_dir_all(&prompts).unwrap();
        let path = prompts.join("deep_capsule.txt");
        let mut f = fs::File::create(&path).unwrap();
        write!(f, "{}", "字".repeat(DEEP_CAPSULE_MAX_CHARS + 1)).unwrap();
        let err = validate_deep_capsule_file(dir.path(), false).unwrap_err();
        assert!(err[0].contains("超过"));
    }

    #[test]
    fn accepts_valid_capsule() {
        let dir = tempfile::tempdir().unwrap();
        let prompts = dir.path().join("prompts");
        fs::create_dir_all(&prompts).unwrap();
        fs::write(
            prompts.join("deep_capsule.txt"),
            "沐沐：害羞嘴硬的可爱小女孩。",
        )
        .unwrap();
        validate_deep_capsule_file(dir.path(), true).unwrap();
    }
}

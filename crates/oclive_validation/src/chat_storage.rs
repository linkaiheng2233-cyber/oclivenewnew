//! `config.json` → `chat_storage` validation.

/// Validate `chat_storage` JSON object (from parsed `config.json`).
#[must_use]
pub fn validate_chat_storage_config(value: &serde_json::Value) -> Vec<String> {
    let mut errs = Vec::new();

    if let Some(backend) = value.get("backend").and_then(|v| v.as_str()) {
        let b = backend.trim().to_ascii_lowercase();
        if !matches!(b.as_str(), "hybrid" | "file" | "sqlite") {
            errs.push(format!(
                "config.json chat_storage.backend 须为 hybrid | file | sqlite（当前「{backend}」）"
            ));
        }
    }

    if let Some(mirror) = value.get("mirror") {
        if !mirror.is_boolean() {
            errs.push("config.json chat_storage.mirror 须为 bool".into());
        }
    }

    if let Some(location) = value.get("location").and_then(|v| v.as_str()) {
        let loc = location.trim().to_ascii_lowercase();
        if !matches!(loc.as_str(), "global" | "role_pack") {
            errs.push(format!(
                "config.json chat_storage.location 须为 global | role_pack（当前「{location}」）"
            ));
        }
    }

    for key in ["max_messages_per_session", "auto_cleanup_days", "auto_cleanup_max_sessions"] {
        if let Some(v) = value.get(key) {
            match v.as_u64() {
                Some(n) if n >= 1 => {}
                _ => errs.push(format!(
                    "config.json chat_storage.{key} 须为正整数"
                )),
            }
        }
    }

    if let Some(v) = value.get("replay_similarity_threshold") {
        match v.as_f64() {
            Some(n) if (0.1..=1.0).contains(&n) => {}
            _ => errs.push(
                "config.json chat_storage.replay_similarity_threshold 须在 0.1–1.0 范围内".into(),
            ),
        }
    }

    errs
}

/// Validate optional `config.json` on disk.
///
/// # Errors
///
/// Returns validation error strings when `chat_storage` section is invalid.
pub fn validate_chat_storage_config_file(
    config_path: &std::path::Path,
) -> Result<(), Vec<String>> {
    if !config_path.is_file() {
        return Ok(());
    }
    let raw = std::fs::read_to_string(config_path)
        .map_err(|e| vec![format!("读取 {} 失败: {}", config_path.display(), e)])?;
    let root: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| vec![format!("解析 {} 失败: {}", config_path.display(), e)])?;
    if let Some(section) = root.get("chat_storage") {
        let errs = validate_chat_storage_config(section);
        if errs.is_empty() {
            Ok(())
        } else {
            Err(errs)
        }
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn accepts_hybrid_defaults() {
        let v = json!({
            "backend": "hybrid",
            "location": "global",
            "replay_similarity_threshold": 0.6
        });
        assert!(validate_chat_storage_config(&v).is_empty());
    }

    #[test]
    fn rejects_unknown_backend() {
        let v = json!({ "backend": "postgres" });
        let errs = validate_chat_storage_config(&v);
        assert!(errs.iter().any(|e| e.contains("backend")));
    }

    #[test]
    fn rejects_threshold_out_of_range() {
        let v = json!({ "replay_similarity_threshold": 0.05 });
        let errs = validate_chat_storage_config(&v);
        assert!(errs.iter().any(|e| e.contains("replay_similarity_threshold")));
    }

    #[test]
    fn missing_section_fields_pass() {
        assert!(validate_chat_storage_config(&json!({})).is_empty());
    }
}

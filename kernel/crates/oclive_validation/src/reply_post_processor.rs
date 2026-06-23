//! `config.json` → `reply_post_processor` validation.

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
struct ReplyPostProcessorConfigDisk {
    #[serde(default)]
    enabled: bool,
    #[serde(default = "default_backend")]
    backend: String,
}

fn default_backend() -> String {
    "builtin".to_string()
}

/// Validate `reply_post_processor` JSON object (from parsed `config.json`).
#[must_use]
pub fn validate_reply_post_processor_config(value: &serde_json::Value) -> Vec<String> {
    let mut errs = Vec::new();
    let cfg: ReplyPostProcessorConfigDisk = match serde_json::from_value(value.clone()) {
        Ok(c) => c,
        Err(e) => {
            errs.push(format!("config.json reply_post_processor 解析失败: {e}"));
            return errs;
        }
    };

    let backend = cfg.backend.trim().to_ascii_lowercase();
    if !matches!(backend.as_str(), "builtin" | "remote" | "directory") {
        errs.push(format!(
            "config.json reply_post_processor.backend 须为 builtin | remote | directory（当前「{}」）",
            cfg.backend
        ));
    }

    if cfg.enabled && backend == "remote" {
        if let Some(url) = value
            .get("remote")
            .and_then(|r| r.get("url"))
            .and_then(|u| u.as_str())
        {
            if url.trim().is_empty() {
                errs.push(
                    "config.json reply_post_processor: backend=remote 且 enabled 时 remote.url 必填非空"
                        .into(),
                );
            }
        } else {
            errs.push(
                "config.json reply_post_processor: backend=remote 且 enabled 时 remote.url 必填非空"
                    .into(),
            );
        }
    }

    if cfg.enabled && backend == "directory" {
        if let Some(plugin_id) = value
            .get("directory")
            .and_then(|d| d.get("plugin_id"))
            .and_then(|p| p.as_str())
        {
            if plugin_id.trim().is_empty() {
                errs.push(
                    "config.json reply_post_processor: backend=directory 且 enabled 时 directory.plugin_id 必填非空"
                        .into(),
                );
            }
        } else {
            errs.push(
                "config.json reply_post_processor: backend=directory 且 enabled 时 directory.plugin_id 必填非空"
                    .into(),
            );
        }
    }

    if let Some(profile) = value
        .get("builtin")
        .and_then(|b| b.get("profile"))
        .and_then(|p| p.as_str())
    {
        let p = profile.trim().to_ascii_lowercase();
        if !matches!(p.as_str(), "standard" | "minimal") {
            errs.push(format!(
                "config.json reply_post_processor.builtin.profile 须为 standard | minimal（当前「{profile}」）"
            ));
        }
    }

    errs
}

/// Validate optional `config.json` on disk.
///
/// # Errors
///
/// Returns validation error strings when `reply_post_processor` section is invalid.
pub fn validate_reply_post_processor_config_file(
    config_path: &std::path::Path,
) -> Result<(), Vec<String>> {
    if !config_path.is_file() {
        return Ok(());
    }
    let raw = std::fs::read_to_string(config_path)
        .map_err(|e| vec![format!("读取 {} 失败: {}", config_path.display(), e)])?;
    let root: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| vec![format!("解析 {} 失败: {}", config_path.display(), e)])?;
    if let Some(section) = root.get("reply_post_processor") {
        let errs = validate_reply_post_processor_config(section);
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
    fn rejects_unknown_backend() {
        let v = json!({ "backend": "wasm" });
        let errs = validate_reply_post_processor_config(&v);
        assert!(errs.iter().any(|e| e.contains("backend")));
    }

    #[test]
    fn accepts_builtin_minimal_profile() {
        let v = json!({
            "enabled": false,
            "backend": "builtin",
            "builtin": { "profile": "minimal" }
        });
        assert!(validate_reply_post_processor_config(&v).is_empty());
    }

    #[test]
    fn remote_enabled_requires_url() {
        let v = json!({ "enabled": true, "backend": "remote", "remote": { "url": "" } });
        let errs = validate_reply_post_processor_config(&v);
        assert!(errs.iter().any(|e| e.contains("remote.url")));
    }

    #[test]
    fn directory_enabled_requires_plugin_id() {
        let v =
            json!({ "enabled": true, "backend": "directory", "directory": { "plugin_id": "" } });
        let errs = validate_reply_post_processor_config(&v);
        assert!(errs.iter().any(|e| e.contains("plugin_id")));
    }
}

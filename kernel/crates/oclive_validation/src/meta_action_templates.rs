//! `config.json` → `meta_action_templates` validation.

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Default)]
struct MetaActionTemplateEntryDisk {
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    attitude_text: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct MetaActionTemplatesDisk {
    #[serde(default)]
    undo: MetaActionTemplateEntryDisk,
    #[serde(default)]
    regenerate: MetaActionTemplateEntryDisk,
    #[serde(default)]
    edit: MetaActionTemplateEntryDisk,
    #[serde(default)]
    delete: MetaActionTemplateEntryDisk,
}

fn validate_entry(name: &str, entry: &MetaActionTemplateEntryDisk) -> Vec<String> {
    let mut errs = Vec::new();
    if entry.enabled && entry.attitude_text.len() > 2000 {
        errs.push(format!(
            "config.json meta_action_templates.{name}.attitude_text 过长（上限 2000 字符）"
        ));
    }
    errs
}

/// Validate `meta_action_templates` JSON object (from parsed `config.json`).
#[must_use]
pub fn validate_meta_action_templates_config(value: &serde_json::Value) -> Vec<String> {
    let mut errs = Vec::new();
    let cfg: MetaActionTemplatesDisk = match serde_json::from_value(value.clone()) {
        Ok(c) => c,
        Err(e) => {
            errs.push(format!("config.json meta_action_templates 解析失败: {e}"));
            return errs;
        }
    };
    errs.extend(validate_entry("undo", &cfg.undo));
    errs.extend(validate_entry("regenerate", &cfg.regenerate));
    errs.extend(validate_entry("edit", &cfg.edit));
    errs.extend(validate_entry("delete", &cfg.delete));
    errs
}

/// Validate optional `config.json` on disk.
///
/// # Errors
///
/// Returns validation error strings when `meta_action_templates` section is invalid.
pub fn validate_meta_action_templates_config_file(
    config_path: &std::path::Path,
) -> Result<(), Vec<String>> {
    if !config_path.is_file() {
        return Ok(());
    }
    let raw = std::fs::read_to_string(config_path)
        .map_err(|e| vec![format!("读取 {} 失败: {}", config_path.display(), e)])?;
    let root: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| vec![format!("解析 {} 失败: {}", config_path.display(), e)])?;
    if let Some(section) = root.get("meta_action_templates") {
        let errs = validate_meta_action_templates_config(section);
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
    fn accepts_minimal_templates() {
        let v = json!({
            "undo": { "enabled": true, "attitude_text": "用户撤回了上一轮。" },
            "regenerate": { "enabled": false, "attitude_text": "" }
        });
        assert!(validate_meta_action_templates_config(&v).is_empty());
    }

    #[test]
    fn rejects_oversized_attitude_text() {
        let v = json!({
            "edit": { "enabled": true, "attitude_text": "x".repeat(2001) }
        });
        let errs = validate_meta_action_templates_config(&v);
        assert!(errs.iter().any(|e| e.contains("attitude_text")));
    }
}

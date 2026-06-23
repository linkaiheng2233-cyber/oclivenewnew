//! `config.json` → `penetration_templates` validation (additive; VS Code extension + pack authors).

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Default)]
struct PenetrationTemplatesDisk {
    #[serde(default)]
    enabled: Option<bool>,
    #[serde(default)]
    diary_header: String,
    #[serde(default)]
    diary_path: String,
    #[serde(default)]
    letter_template: String,
    #[serde(default)]
    idle_message: String,
}

fn validate_optional_string(name: &str, value: &str, max_len: usize) -> Vec<String> {
    let mut errs = Vec::new();
    if value.len() > max_len {
        errs.push(format!(
            "config.json penetration_templates.{name} 过长（上限 {max_len} 字符）"
        ));
    }
    if name == "diary_path" && !value.is_empty() && (value.contains("..") || value.starts_with('/'))
    {
        errs.push("config.json penetration_templates.diary_path 不得含 .. 或绝对路径".into());
    }
    errs
}

/// Validate `penetration_templates` JSON object (from parsed `config.json`).
#[must_use]
pub fn validate_penetration_templates_config(value: &serde_json::Value) -> Vec<String> {
    let mut errs = Vec::new();
    let cfg: PenetrationTemplatesDisk = match serde_json::from_value(value.clone()) {
        Ok(c) => c,
        Err(e) => {
            errs.push(format!("config.json penetration_templates 解析失败: {e}"));
            return errs;
        }
    };
    errs.extend(validate_optional_string(
        "diary_header",
        &cfg.diary_header,
        500,
    ));
    errs.extend(validate_optional_string("diary_path", &cfg.diary_path, 260));
    errs.extend(validate_optional_string(
        "letter_template",
        &cfg.letter_template,
        2000,
    ));
    errs.extend(validate_optional_string(
        "idle_message",
        &cfg.idle_message,
        500,
    ));
    let _ = cfg.enabled;
    errs
}

/// Validate optional `config.json` on disk.
///
/// # Errors
///
/// Returns validation error strings when `penetration_templates` section is invalid.
pub fn validate_penetration_templates_config_file(
    config_path: &std::path::Path,
) -> Result<(), Vec<String>> {
    if !config_path.is_file() {
        return Ok(());
    }
    let raw = std::fs::read_to_string(config_path)
        .map_err(|e| vec![format!("读取 {} 失败: {}", config_path.display(), e)])?;
    let root: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| vec![format!("解析 {} 失败: {}", config_path.display(), e)])?;
    if let Some(section) = root.get("penetration_templates") {
        let errs = validate_penetration_templates_config(section);
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
            "enabled": true,
            "diary_header": "今日片段",
            "idle_message": "回来聊聊？"
        });
        assert!(validate_penetration_templates_config(&v).is_empty());
    }

    #[test]
    fn rejects_traversal_diary_path() {
        let v = json!({ "diary_path": "../secrets.md" });
        let errs = validate_penetration_templates_config(&v);
        assert!(errs.iter().any(|e| e.contains("diary_path")));
    }
}

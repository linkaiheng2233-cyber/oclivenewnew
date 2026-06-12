//! `portrait_catalog.json` + `config.json` → `portrait_catalog.enabled` validation.

use std::path::Path;

const SIMPLE_PORTRAIT_SLOT_IDS: &[&str] = &[
    "happy_default",
    "sad_default",
    "angry_default",
    "neutral_default",
    "excited_default",
    "confused_default",
    "shy_default",
];

/// Validate `portrait_catalog.json` when present; cross-check with config toggle.
pub fn validate_portrait_catalog_files(role_dir: &Path) -> Result<(), Vec<String>> {
    let config_path = role_dir.join("config.json");
    let catalog_path = role_dir.join("portrait_catalog.json");
    let mut errs = Vec::new();

    let enabled = if config_path.is_file() {
        match std::fs::read_to_string(&config_path) {
            Ok(s) => match serde_json::from_str::<serde_json::Value>(&s) {
                Ok(root) => root
                    .get("portrait_catalog")
                    .and_then(|v| v.get("enabled"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                Err(e) => {
                    errs.push(format!("config.json 解析失败: {e}"));
                    false
                }
            },
            Err(e) => {
                errs.push(format!("config.json 不可读: {e}"));
                false
            }
        }
    } else {
        false
    };

    if !catalog_path.is_file() {
        if enabled {
            errs.push(
                "config.json portrait_catalog.enabled=true 但缺少 portrait_catalog.json".into(),
            );
        }
        return if errs.is_empty() { Ok(()) } else { Err(errs) };
    }

    let catalog = match std::fs::read_to_string(&catalog_path) {
        Ok(s) => match serde_json::from_str::<serde_json::Value>(&s) {
            Ok(v) => v,
            Err(e) => {
                errs.push(format!("portrait_catalog.json 解析失败: {e}"));
                return Err(errs);
            }
        },
        Err(e) => {
            errs.push(format!("portrait_catalog.json 不可读: {e}"));
            return Err(errs);
        }
    };

    validate_catalog_assets(role_dir, &catalog, &mut errs);

    if enabled {
        for slot_id in SIMPLE_PORTRAIT_SLOT_IDS {
            if !catalog_assets(&catalog).iter().any(|a| asset_id(a) == Some(*slot_id)) {
                errs.push(format!(
                    "portrait_catalog 启用时缺少简单包固定 id「{slot_id}」"
                ));
            }
        }
    }

    if errs.is_empty() {
        Ok(())
    } else {
        Err(errs)
    }
}

fn catalog_assets(catalog: &serde_json::Value) -> Vec<&serde_json::Value> {
    catalog
        .get("assets")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().collect())
        .unwrap_or_default()
}

fn asset_id(asset: &serde_json::Value) -> Option<&str> {
    asset.get("id").and_then(|v| v.as_str())
}

fn validate_catalog_assets(
    role_dir: &Path,
    catalog: &serde_json::Value,
    errs: &mut Vec<String>,
) {
    let mut seen_ids = std::collections::HashSet::new();
    for asset in catalog_assets(catalog) {
        let Some(id) = asset_id(asset) else {
            errs.push("portrait_catalog.assets[] 缺少 id".into());
            continue;
        };
        if id.trim().is_empty() {
            errs.push("portrait_catalog.assets[] 存在空 id".into());
            continue;
        }
        if !seen_ids.insert(id.to_string()) {
            errs.push(format!("portrait_catalog.assets id 重复: {id}"));
        }
        let Some(path) = asset.get("path").and_then(|v| v.as_str()) else {
            errs.push(format!("portrait_catalog.assets[{id}] 缺少 path"));
            continue;
        };
        if path.trim().is_empty() {
            errs.push(format!("portrait_catalog.assets[{id}] path 为空"));
            continue;
        }
        if path.contains("..") || path.starts_with('/') || path.contains('\\') {
            errs.push(format!("portrait_catalog.assets[{id}] path 不安全: {path}"));
        }
        let full = role_dir.join(path);
        if !full.is_file() {
            errs.push(format!("portrait_catalog.assets[{id}] path 不存在: {path}"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_duplicate_ids() {
        let dir = tempfile::tempdir().unwrap();
        let role = dir.path();
        std::fs::create_dir_all(role.join("assets/images")).unwrap();
        std::fs::write(role.join("assets/images/a.png"), b"x").unwrap();
        let catalog = r#"{"schema_version":1,"assets":[
            {"id":"a","path":"assets/images/a.png","tags":["happy"]},
            {"id":"a","path":"assets/images/a.png","tags":["sad"]}
        ]}"#;
        std::fs::write(role.join("portrait_catalog.json"), catalog).unwrap();
        let errs = validate_portrait_catalog_files(role).unwrap_err();
        assert!(errs.iter().any(|e| e.contains("重复")));
    }
}

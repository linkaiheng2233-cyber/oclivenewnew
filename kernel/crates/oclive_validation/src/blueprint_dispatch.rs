//! Version dispatch for `pipeline.ocblueprint`.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde_json::Value;

use crate::blueprint_v2::{
    SlotRegistryEntry, BLUEPRINT_V2_SCHEMA_VERSION, PIPELINE_BLUEPRINT_FILENAME,
};
use crate::blueprint_v3::BLUEPRINT_V3_SCHEMA_VERSION;
use crate::blueprint_v4::BLUEPRINT_V4_SCHEMA_VERSION;

/// Dispatch blueprint JSON validation by `schema_version`.
///
/// # Errors
///
/// Returns all structural contract failures, or an unsupported-version error.
pub fn validate_blueprint_json_by_schema_version(
    raw: &str,
    folder_name: Option<&str>,
) -> Result<Vec<String>, Vec<String>> {
    let root: Value = serde_json::from_str(raw)
        .map_err(|e| vec![format!("pipeline.ocblueprint JSON 语法错误: {e}")])?;
    let version = root
        .get("schema_version")
        .and_then(Value::as_u64)
        .unwrap_or(0) as u32;
    match version {
        BLUEPRINT_V2_SCHEMA_VERSION => {
            crate::blueprint_v2::validate_blueprint_v2_json(raw)?;
            Ok(warnings_for_v2_runtime_config(&root))
        }
        BLUEPRINT_V3_SCHEMA_VERSION => {
            crate::blueprint_v3::validate_blueprint_v3_json(raw, folder_name)?;
            Ok(Vec::new())
        }
        BLUEPRINT_V4_SCHEMA_VERSION => {
            crate::blueprint_v4::validate_blueprint_v4_json(raw, folder_name)?;
            Ok(Vec::new())
        }
        other => Err(vec![format!(
            "pipeline.ocblueprint：不支持的 schema_version {other}（支持 {BLUEPRINT_V2_SCHEMA_VERSION}、{BLUEPRINT_V3_SCHEMA_VERSION} 或 {BLUEPRINT_V4_SCHEMA_VERSION}）"
        )]),
    }
}

fn warnings_for_v2_runtime_config(root: &Value) -> Vec<String> {
    if root.get("runtime_config").is_some() {
        vec![
            "注意：schema_version 2 下顶层 runtime_config 不参与宿主加载（将被忽略）；稳定蓝图请升级到 schema_version 4，双核 Beta 才使用 schema_version 3"
                .into(),
        ]
    } else {
        Vec::new()
    }
}

/// Read `schema_version` from blueprint JSON (`None` when parsing fails).
#[must_use]
pub fn blueprint_schema_version_from_raw(raw: &str) -> Option<u32> {
    serde_json::from_str::<Value>(raw)
        .ok()
        .and_then(|value| value.get("schema_version").and_then(Value::as_u64))
        .map(|version| version as u32)
}

/// Load only the validated `slot_registry` from any supported blueprint
/// version. CLI management tools use this to avoid maintaining a second
/// version router.
///
/// # Errors
///
/// Returns file, contract, directory, or unsupported-version failures.
pub fn load_blueprint_slot_registry_for_role_dir(
    role_dir: &Path,
    host_version: &str,
) -> Result<BTreeMap<String, SlotRegistryEntry>, Vec<String>> {
    let blueprint_path = role_dir.join(PIPELINE_BLUEPRINT_FILENAME);
    let raw = fs::read_to_string(&blueprint_path)
        .map_err(|error| vec![format!("读取 {} 失败: {error}", blueprint_path.display())])?;
    let version = blueprint_schema_version_from_raw(&raw).unwrap_or(0);
    match version {
        BLUEPRINT_V2_SCHEMA_VERSION => {
            crate::blueprint_v2::load_blueprint_v2_for_role_dir(role_dir, host_version)
                .map(|loaded| loaded.slot_registry)
        }
        BLUEPRINT_V3_SCHEMA_VERSION => {
            crate::blueprint_v3::load_blueprint_v3_for_role_dir(role_dir, host_version)
                .map(|loaded| loaded.slot_registry)
        }
        BLUEPRINT_V4_SCHEMA_VERSION => {
            crate::blueprint_v4::load_blueprint_v4_for_role_dir(role_dir, host_version)
                .map(|loaded| loaded.slot_registry)
        }
        unsupported => Err(vec![format!(
            "pipeline.ocblueprint：不支持的 schema_version {unsupported}（支持 {BLUEPRINT_V2_SCHEMA_VERSION}、{BLUEPRINT_V3_SCHEMA_VERSION} 或 {BLUEPRINT_V4_SCHEMA_VERSION}）"
        )]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unknown_schema_version_with_supported_versions() {
        let errors =
            validate_blueprint_json_by_schema_version(r#"{"schema_version":9}"#, None).unwrap_err();
        assert!(errors[0].contains("2、3 或 4"));
    }

    #[test]
    fn v2_runtime_config_warning_points_to_stable_v4() {
        let raw = r#"{
          "schema_version": 2,
          "meta": {
            "id": "demo",
            "name": "Demo",
            "version": "1.0.0",
            "author": "A",
            "description": "D",
            "relations": {"friend": {"display_name": "F", "initial_favorability": 50, "favor_multiplier": 1}},
            "default_relation": "friend"
          },
          "slot_registry": {
            "llm": {"type": "llm", "label": "LLM", "backend": "ollama", "position": 0}
          },
          "runtime_config": {}
        }"#;
        let warnings = validate_blueprint_json_by_schema_version(raw, None).unwrap();
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("schema_version 4"));
    }
}

//! 角色包蓝图（`pipeline.ocblueprint` v2）读取与校验。

use anyhow::{Context, Result};
use oclive_validation::validate_blueprint_v2_json;
use serde_json::Value;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationReport {
    pub ok: bool,
    pub errors: Vec<ValidationError>,
}

impl ValidationReport {
    pub fn success() -> Self {
        Self {
            ok: true,
            errors: vec![],
        }
    }

    pub fn from_strings(errs: Vec<String>) -> Self {
        if errs.is_empty() {
            Self::success()
        } else {
            Self {
                ok: false,
                errors: errs
                    .into_iter()
                    .map(|message| ValidationError { message })
                    .collect(),
            }
        }
    }
}

/// 从磁盘读取蓝图 JSON 文本。
pub fn load_blueprint_text(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("read blueprint: {}", path.display()))
}

/// 校验 `pipeline.ocblueprint`（仅 schema_version 2；废弃 steps[] DSL）。
pub fn validate_blueprint_file(raw: &str) -> ValidationReport {
    let v: Value = match serde_json::from_str(raw) {
        Ok(x) => x,
        Err(e) => {
            return ValidationReport::from_strings(vec![format!("parse blueprint JSON: {}", e)]);
        }
    };

    if let Value::Object(map) = &v {
        if map.contains_key("steps") {
            return ValidationReport::from_strings(vec![
                "legacy pipeline DSL (steps[]/entry) is deprecated; use schema_version 2 with meta + slot_registry".into(),
            ]);
        }
        if let Some(Value::Number(n)) = map.get("schema_version") {
            if n.as_u64() != Some(2) {
                return ValidationReport::from_strings(vec![format!(
                    "unsupported schema_version {} (only integer 2 is accepted)",
                    n
                )]);
            }
        } else {
            return ValidationReport::from_strings(vec![
                "pipeline.ocblueprint must include schema_version: 2".into(),
            ]);
        }
    }

    ValidationReport::from_strings(
        validate_blueprint_v2_json(raw).err().unwrap_or_default(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL: &str = r#"{
      "schema_version": 2,
      "meta": {
        "id": "demo",
        "name": "Demo",
        "version": "0.1.0",
        "author": "t",
        "description": "d",
        "personality": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5],
        "relations": {
          "friend": { "initial_favorability": 50.0, "favor_multiplier": 1.0 }
        },
        "default_relation": "friend"
      },
      "slot_registry": {
        "llm": { "type": "llm", "label": "LLM", "backend": "ollama", "position": 0 }
      }
    }"#;

    #[test]
    fn v2_passes() {
        assert!(validate_blueprint_file(MINIMAL).ok);
    }

    #[test]
    fn legacy_steps_rejected() {
        let raw = r#"{"schema_version":1,"steps":[{"id":"s1","type":"call_llm"}]}"#;
        assert!(!validate_blueprint_file(raw).ok);
    }
}

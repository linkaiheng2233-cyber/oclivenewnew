//! Reading and version-dispatched validation of role pack blueprints.

use anyhow::{Context, Result};
use oclive_validation::validate_blueprint_json_by_schema_version;
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

/// Reads the blueprint JSON text from disk.
pub fn load_blueprint_text(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("read blueprint: {}", path.display()))
}

/// Validates `pipeline.ocblueprint` using the v2 / frozen-v3 / Stable-v4 contract.
pub fn validate_blueprint_file(raw: &str) -> ValidationReport {
    match validate_blueprint_json_by_schema_version(raw, None) {
        Ok(_) => ValidationReport::success(),
        Err(errors) => ValidationReport::from_strings(errors),
    }
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

    #[test]
    fn stable_v4_passes() {
        let raw = MINIMAL.replacen("\"schema_version\": 2", "\"schema_version\": 4", 1);
        assert!(validate_blueprint_file(&raw).ok);
    }

    #[test]
    fn unknown_version_is_not_treated_as_v2() {
        let raw = MINIMAL.replacen("\"schema_version\": 2", "\"schema_version\": 99", 1);
        let report = validate_blueprint_file(&raw);
        assert!(!report.ok);
        assert!(report.errors[0].message.contains("schema_version 99"));
    }
}

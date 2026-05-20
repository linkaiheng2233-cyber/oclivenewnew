//! 角色包蓝图（`pipeline.ocblueprint`）读取与校验（纯内核定制预备工具链）。

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// 内核已知编排步骤类型（与 docs/BLUEPRINT_REFERENCE.md 一致）。
pub const KNOWN_STEP_TYPES: &[&str] = &[
    "load_context",
    "analyze_emotion",
    "detect_event",
    "retrieve_memory",
    "build_prompt",
    "call_llm",
    "post_process",
];

#[derive(Debug, Clone, Deserialize)]
pub struct BlueprintFile {
    #[serde(default)]
    pub schema_version: u32,
    pub entry: Option<String>,
    pub steps: Vec<BlueprintStep>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BlueprintStep {
    pub id: String,
    #[serde(rename = "type")]
    pub step_type: String,
    #[serde(default)]
    pub next: Option<String>,
}

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
}

/// 从磁盘读取蓝图 JSON（`.ocblueprint` / `.json`）。
pub fn load_blueprint(path: &Path) -> Result<BlueprintFile> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("read blueprint: {}", path.display()))?;
    let v: Value = serde_json::from_str(&text).context("parse blueprint JSON")?;
    if !v.is_object() {
        anyhow::bail!("blueprint root must be a JSON object");
    }
    serde_json::from_value(v).context("deserialize blueprint")
}

/// 校验步骤 `id`/`type`/`next` 引用与已知类型集合。
pub fn validate_blueprint(bp: &BlueprintFile) -> ValidationReport {
    let mut errors = Vec::new();
    let known_types: HashSet<&str> = KNOWN_STEP_TYPES.iter().copied().collect();

    if bp.steps.is_empty() {
        errors.push(ValidationError {
            message: "steps must not be empty".into(),
        });
    }

    let mut ids = HashSet::new();
    for (i, step) in bp.steps.iter().enumerate() {
        if step.id.trim().is_empty() {
            errors.push(ValidationError {
                message: format!("steps[{i}].id must be non-empty"),
            });
        } else if !ids.insert(step.id.clone()) {
            errors.push(ValidationError {
                message: format!("duplicate step id: {}", step.id),
            });
        }
        if step.step_type.trim().is_empty() {
            errors.push(ValidationError {
                message: format!("steps[{i}].type must be non-empty"),
            });
        } else if !known_types.contains(step.step_type.as_str()) {
            errors.push(ValidationError {
                message: format!(
                    "steps[{i}].type unknown: {} (known: {})",
                    step.step_type,
                    KNOWN_STEP_TYPES.join(", ")
                ),
            });
        }
        if let Some(ref next) = step.next {
            if next.trim().is_empty() {
                errors.push(ValidationError {
                    message: format!("steps[{i}].next must be non-empty when present"),
                });
            }
        }
    }

    for (i, step) in bp.steps.iter().enumerate() {
        if let Some(ref next) = step.next {
            if !ids.contains(next) {
                errors.push(ValidationError {
                    message: format!("steps[{i}].next references missing id: {next}"),
                });
            }
        }
    }

    if let Some(ref entry) = bp.entry {
        if !ids.contains(entry) {
            errors.push(ValidationError {
                message: format!("entry references missing step id: {entry}"),
            });
        }
    }

    if errors.is_empty() {
        ValidationReport::success()
    } else {
        ValidationReport { ok: false, errors }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_blueprint_passes() {
        let bp = BlueprintFile {
            schema_version: 1,
            entry: Some("s1".into()),
            steps: vec![
                BlueprintStep {
                    id: "s1".into(),
                    step_type: "load_context".into(),
                    next: Some("s2".into()),
                },
                BlueprintStep {
                    id: "s2".into(),
                    step_type: "call_llm".into(),
                    next: None,
                },
            ],
        };
        assert!(validate_blueprint(&bp).ok);
    }

    #[test]
    fn unknown_type_fails() {
        let bp = BlueprintFile {
            schema_version: 1,
            entry: None,
            steps: vec![BlueprintStep {
                id: "x".into(),
                step_type: "not_a_step".into(),
                next: None,
            }],
        };
        assert!(!validate_blueprint(&bp).ok);
    }
}

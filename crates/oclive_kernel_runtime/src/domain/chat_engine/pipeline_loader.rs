//! 角色目录下 `pipeline.ocblueprint` 的发现、解析与校验。

use serde::Deserialize;
use std::path::{Path, PathBuf};

/// 运行时从 `{roles_dir}/{role_id}/pipeline.ocblueprint` 读取。
pub const PIPELINE_BLUEPRINT_FILENAME: &str = "pipeline.ocblueprint";

/// 当前唯一支持的 `schemaVersion`。
pub const SCHEMA_VERSION_V1: &str = "1.0";

/// 单蓝图 `steps` 上限。
pub const MAX_PIPELINE_STEPS: usize = 64;

/// 入口蓝图允许的原子（不含 `validate_scene`，该步在加载蓝图之前已执行）。
pub const ALLOWED_PIPELINE_BLUEPRINT_ACTIONS: &[&str] = &[
    "init_turn",
    "ensure_role_runtime",
    "load_role",
    "seed_interaction_mode",
    "log_effective_plugin_backends",
    "resolve_plugins",
    "resolve_main_llm_model",
    "run_agent",
    "set_user_presence_scene",
    "load_presence_routing",
    "analyze_emotion_user",
];

/// 某步失败时的策略（与 JSON `onFailure` 对应）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OnFailurePolicy {
    #[default]
    Halt,
    Degrade,
}

/// 已校验、可交给解释器的蓝图。
#[derive(Debug, Clone)]
pub struct PipelineBlueprint {
    pub schema_version: String,
    pub name: String,
    pub on_failure: OnFailurePolicy,
    pub steps: Vec<PipelineStepSpec>,
}

#[derive(Debug, Clone)]
pub struct PipelineStepSpec {
    pub action: String,
    pub id: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum BlueprintError {
    #[error("read blueprint file: {0}")]
    Io(#[from] std::io::Error),
    #[error("parse blueprint JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unsupported schemaVersion: {0:?} (expected {1:?})")]
    UnsupportedSchemaVersion(String, &'static str),
    #[error("blueprint name is empty")]
    EmptyName,
    #[error("blueprint steps is empty")]
    EmptySteps,
    #[error("too many steps: {0} (max {1})")]
    TooManySteps(usize, usize),
    #[error("unknown or disallowed action at step {0}: {1:?}")]
    UnknownOrDisallowedAction(usize, String),
    #[error("invalid onFailure: {0:?} (use HALT or DEGRADE)")]
    InvalidOnFailure(String),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PipelineBlueprintFile {
    schema_version: String,
    name: String,
    #[serde(default)]
    on_failure: Option<String>,
    steps: Vec<PipelineStepFile>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PipelineStepFile {
    action: String,
    #[serde(default)]
    id: Option<String>,
}

fn action_allowed_for_blueprint(action: &str) -> bool {
    ALLOWED_PIPELINE_BLUEPRINT_ACTIONS
        .iter()
        .any(|a| *a == action)
}

fn parse_on_failure(raw: Option<&str>) -> Result<OnFailurePolicy, BlueprintError> {
    let opt = raw.map(str::trim).filter(|s| !s.is_empty());
    match opt {
        None => Ok(OnFailurePolicy::Halt),
        Some(s) if s.eq_ignore_ascii_case("HALT") => Ok(OnFailurePolicy::Halt),
        Some(s) if s.eq_ignore_ascii_case("DEGRADE") => Ok(OnFailurePolicy::Degrade),
        Some(other) => Err(BlueprintError::InvalidOnFailure(other.to_string())),
    }
}

/// 从字节解析并校验（测试与工具复用）。
pub fn parse_and_validate_blueprint_bytes(bytes: &[u8]) -> Result<PipelineBlueprint, BlueprintError> {
    let file: PipelineBlueprintFile = serde_json::from_slice(bytes)?;
    validate_file(file)
}

fn validate_file(file: PipelineBlueprintFile) -> Result<PipelineBlueprint, BlueprintError> {
    let sv = file.schema_version.trim();
    if sv != SCHEMA_VERSION_V1 {
        return Err(BlueprintError::UnsupportedSchemaVersion(
            file.schema_version.clone(),
            SCHEMA_VERSION_V1,
        ));
    }
    let name = file.name.trim().to_string();
    if name.is_empty() {
        return Err(BlueprintError::EmptyName);
    }
    if file.steps.is_empty() {
        return Err(BlueprintError::EmptySteps);
    }
    if file.steps.len() > MAX_PIPELINE_STEPS {
        return Err(BlueprintError::TooManySteps(file.steps.len(), MAX_PIPELINE_STEPS));
    }
    let on_failure = parse_on_failure(file.on_failure.as_deref())?;
    let mut steps = Vec::with_capacity(file.steps.len());
    for (idx, s) in file.steps.into_iter().enumerate() {
        let action = s.action.trim().to_string();
        if !action_allowed_for_blueprint(action.as_str()) {
            return Err(BlueprintError::UnknownOrDisallowedAction(idx, action));
        }
        steps.push(PipelineStepSpec {
            action,
            id: s.id.map(|x| x.trim().to_string()).filter(|x| !x.is_empty()),
        });
    }
    Ok(PipelineBlueprint {
        schema_version: sv.to_string(),
        name,
        on_failure,
        steps,
    })
}

/// 从路径读取；文件不存在则 `Ok(None)`。
pub fn load_blueprint_from_path(path: &Path) -> Result<Option<PipelineBlueprint>, BlueprintError> {
    if !path.is_file() {
        return Ok(None);
    }
    let bytes = std::fs::read(path)?;
    Ok(Some(parse_and_validate_blueprint_bytes(&bytes)?))
}

/// `{roles_dir}/{manifest_role_id}/pipeline.ocblueprint`。
pub fn blueprint_path_for_role(roles_dir: &Path, manifest_role_id: &str) -> PathBuf {
    roles_dir
        .join(manifest_role_id)
        .join(PIPELINE_BLUEPRINT_FILENAME)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_repo_example_simple_companion() {
        let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/blueprints/simple_companion.ocblueprint");
        let bytes = std::fs::read(&p).expect("read example blueprint");
        let bp = parse_and_validate_blueprint_bytes(&bytes).expect("valid");
        assert_eq!(bp.schema_version, SCHEMA_VERSION_V1);
        assert_eq!(bp.name, "simple_companion");
        assert_eq!(bp.steps.len(), 8);
        assert_eq!(bp.on_failure, OnFailurePolicy::Halt);
    }

    #[test]
    fn rejects_unknown_action() {
        let j = br#"{"schemaVersion":"1.0","name":"x","steps":[{"action":"not_an_atom"}]}"#;
        let e = parse_and_validate_blueprint_bytes(j).unwrap_err();
        assert!(matches!(e, BlueprintError::UnknownOrDisallowedAction(0, _)));
    }

    #[test]
    fn rejects_bad_on_failure() {
        let j = br#"{"schemaVersion":"1.0","name":"x","onFailure":"PANIC","steps":[{"action":"init_turn"}]}"#;
        let e = parse_and_validate_blueprint_bytes(j).unwrap_err();
        assert!(matches!(e, BlueprintError::InvalidOnFailure(_)));
    }
}

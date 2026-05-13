//! 角色目录下 `pipeline.ocblueprint` 的发现、解析与校验。

use super::pipeline_predicates::PipelinePredicate;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// 运行时从 `{roles_dir}/{role_id}/pipeline.ocblueprint` 读取。
pub const PIPELINE_BLUEPRINT_FILENAME: &str = "pipeline.ocblueprint";

/// 当前唯一支持的 `schemaVersion`。
pub const SCHEMA_VERSION_V1: &str = "1.0";

/// 根 `steps` 数组长度上限。
pub const MAX_PIPELINE_ROOT_STEPS: usize = 64;

/// 整棵树（含 `branch` / `parallel` 子步骤）节点数上限。
pub const MAX_PIPELINE_TREE_NODES: usize = 200;

/// `branch` / `parallel` 控制流最大嵌套深度（根 `steps` 为第 0 层；进入子 `branch` / `parallel` 子树时深度 +1）。
///
/// 与「循环」约束：v0 蓝图为树形 JSON，无跨步 `goto`；无限展开由本上限与 `MAX_PIPELINE_TREE_NODES` 共同保证。
/// 若未来引入跨步引用，须在此补充显式环检测。
pub const MAX_PIPELINE_CONTROL_FLOW_NEST_DEPTH: usize = 3;

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
    "memory_retrieve_short_term",
    "memory_retrieve_long_term",
    "assemble_prompt",
    "generate_response",
    "expert_empathy_touch",
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
pub struct PipelineBranchSpec {
    pub predicate: PipelinePredicate,
    pub on_true: Vec<PipelineStepSpec>,
    pub on_false: Vec<PipelineStepSpec>,
}

#[derive(Debug, Clone)]
pub struct PipelineStepSpec {
    pub id: Option<String>,
    /// 线性原子；与 `branch` / `parallel` 互斥。
    pub action: Option<String>,
    pub branch: Option<PipelineBranchSpec>,
    /// 与 `branch` 互斥；值为多个 **arm**，每个 arm 为线性步骤列表（可嵌套子 `parallel`，不可含 `branch`）。
    pub parallel: Option<Vec<Vec<PipelineStepSpec>>>,
}

#[derive(Debug, thiserror::Error)]
pub enum BlueprintError {
    #[error("[PIPELINE_LOAD_IO] read blueprint file: {0}")]
    Io(#[from] std::io::Error),
    #[error("[PIPELINE_PARSE_ERROR] parse blueprint JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("[PIPELINE_SCHEMA_VERSION] unsupported schemaVersion {0:?} (expected {1:?})")]
    UnsupportedSchemaVersion(String, &'static str),
    #[error("[PIPELINE_VALIDATION_ERROR] blueprint name is empty")]
    EmptyName,
    #[error("[PIPELINE_VALIDATION_ERROR] blueprint steps is empty")]
    EmptySteps,
    #[error("[PIPELINE_VALIDATION_ERROR] too many root steps: {0} (max {1})")]
    TooManyRootSteps(usize, usize),
    #[error("[PIPELINE_ACTION_NOT_ALLOWED] unknown or disallowed action at {0}: {1:?}")]
    UnknownOrDisallowedAction(String, String),
    #[error("[PIPELINE_VALIDATION_ERROR] invalid onFailure: {0:?} (use HALT or DEGRADE)")]
    InvalidOnFailure(String),
    #[error("[PIPELINE_VALIDATION_ERROR] step {0}: branch and parallel are mutually exclusive")]
    BranchAndParallel(String),
    #[error("[PIPELINE_VALIDATION_ERROR] step {0}: branch step must not set non-empty action")]
    BranchWithAction(String),
    #[error("[PIPELINE_VALIDATION_ERROR] step {0}: linear step requires non-empty action")]
    LinearRequiresAction(String),
    #[error(
        "[PIPELINE_MAX_NESTING_DEPTH] step {0}: BRANCH/PARALLEL nesting exceeds max depth {1}"
    )]
    MaxNestingDepth(String, usize),
    #[error("[PIPELINE_TOO_MANY_NODES] blueprint tree exceeds max nodes ({0})")]
    TooManyNodes(usize),
    #[error("[PIPELINE_PARALLEL_INVALID] step {0}: parallel arm contains branch (not allowed)")]
    ParallelContainsBranch(String),
    #[error("[PIPELINE_PARALLEL_INVALID] step {0}: parallel arm contains WRITE action {1:?}")]
    ParallelContainsWrite(String, String),
    #[error("[PIPELINE_DUPLICATE_STEP_ID] duplicate non-empty step id {id:?} at {second_path:?} (first at {first_path:?})")]
    DuplicateStepId {
        id: String,
        first_path: String,
        second_path: String,
    },
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
struct PipelineBranchFile {
    predicate: PipelinePredicate,
    #[serde(default)]
    on_true: Vec<PipelineStepFile>,
    #[serde(default)]
    on_false: Vec<PipelineStepFile>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PipelineStepFile {
    #[serde(default)]
    action: Option<String>,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    branch: Option<PipelineBranchFile>,
    #[serde(default)]
    parallel: Option<Vec<Vec<PipelineStepFile>>>,
}

fn action_allowed_for_blueprint(action: &str) -> bool {
    ALLOWED_PIPELINE_BLUEPRINT_ACTIONS.contains(&action)
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

fn step_path(prefix: &str, idx: usize) -> String {
    if prefix.is_empty() {
        format!("[{idx}]")
    } else {
        format!("{prefix}[{idx}]")
    }
}

fn count_nodes_in_step(s: &PipelineStepSpec) -> usize {
    let mut n = 1usize;
    if let Some(b) = &s.branch {
        for c in &b.on_true {
            n += count_nodes_in_step(c);
        }
        for c in &b.on_false {
            n += count_nodes_in_step(c);
        }
    }
    if let Some(arms) = &s.parallel {
        for arm in arms {
            for c in arm {
                n += count_nodes_in_step(c);
            }
        }
    }
    n
}

fn count_nodes(steps: &[PipelineStepSpec]) -> usize {
    steps.iter().map(count_nodes_in_step).sum()
}

fn register_step_id(
    id_opt: &Option<String>,
    path: &str,
    seen: &mut HashMap<String, String>,
) -> Result<(), BlueprintError> {
    let Some(id) = id_opt.as_deref().map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(());
    };
    let id = id.to_string();
    if let Some(first_path) = seen.insert(id.clone(), path.to_string()) {
        return Err(BlueprintError::DuplicateStepId {
            id,
            first_path,
            second_path: path.to_string(),
        });
    }
    Ok(())
}

/// 全树非空 `id` 唯一；重复 id 视为编排层面的「循环 / 冲突」风险并在加载期拒绝。
fn validate_unique_step_ids(
    steps: &[PipelineStepSpec],
    prefix: &str,
    seen: &mut HashMap<String, String>,
) -> Result<(), BlueprintError> {
    for (idx, s) in steps.iter().enumerate() {
        let p = step_path(prefix, idx);
        register_step_id(&s.id, &p, seen)?;
        if let Some(b) = &s.branch {
            validate_unique_step_ids(&b.on_true, &format!("{p}.branch.onTrue"), seen)?;
            validate_unique_step_ids(&b.on_false, &format!("{p}.branch.onFalse"), seen)?;
        }
        if let Some(arms) = &s.parallel {
            for (ai, arm) in arms.iter().enumerate() {
                validate_unique_step_ids(arm, &format!("{p}.parallel[{ai}]"), seen)?;
            }
        }
    }
    Ok(())
}

/// 二次遍历：确保每个线性 `action` 均在 [`ALLOWED_PIPELINE_BLUEPRINT_ACTIONS`] 中（与 `ingest_step` 一致，便于维护时防遗漏）。
fn validate_whitelisted_actions_tree(
    steps: &[PipelineStepSpec],
    prefix: &str,
) -> Result<(), BlueprintError> {
    for (idx, s) in steps.iter().enumerate() {
        let p = step_path(prefix, idx);
        if let Some(a) = s.action.as_deref() {
            if !action_allowed_for_blueprint(a) {
                return Err(BlueprintError::UnknownOrDisallowedAction(p, a.to_string()));
            }
        }
        if let Some(b) = &s.branch {
            validate_whitelisted_actions_tree(&b.on_true, &format!("{p}.branch.onTrue"))?;
            validate_whitelisted_actions_tree(&b.on_false, &format!("{p}.branch.onFalse"))?;
        }
        if let Some(arms) = &s.parallel {
            for (ai, arm) in arms.iter().enumerate() {
                validate_whitelisted_actions_tree(arm, &format!("{p}.parallel[{ai}]"))?;
            }
        }
    }
    Ok(())
}

fn validate_parallel_step_node(s: &PipelineStepSpec, path: &str) -> Result<(), BlueprintError> {
    if s.branch.is_some() {
        return Err(BlueprintError::ParallelContainsBranch(path.to_string()));
    }
    if let Some(arms) = &s.parallel {
        for (ai, arm) in arms.iter().enumerate() {
            validate_parallel_arm_readonly(arm, &format!("{path}.parallel[{ai}]"))?;
        }
        return Ok(());
    }
    let a = s
        .action
        .as_deref()
        .ok_or_else(|| BlueprintError::LinearRequiresAction(path.to_string()))?;
    match super::pipeline_actions::action_io_type(a) {
        Some(super::pipeline_actions::ActionIOType::ReadOnly) => Ok(()),
        Some(super::pipeline_actions::ActionIOType::Write) => Err(
            BlueprintError::ParallelContainsWrite(path.to_string(), a.to_string()),
        ),
        None => Err(BlueprintError::UnknownOrDisallowedAction(
            path.to_string(),
            a.to_string(),
        )),
    }
}

fn validate_parallel_arm_readonly(
    steps: &[PipelineStepSpec],
    path: &str,
) -> Result<(), BlueprintError> {
    for (i, s) in steps.iter().enumerate() {
        validate_parallel_step_node(s, &step_path(path, i))?;
    }
    Ok(())
}

fn ingest_branch(
    b: PipelineBranchFile,
    child_depth: usize,
    path: &str,
) -> Result<PipelineBranchSpec, BlueprintError> {
    let on_true = ingest_steps_inner(b.on_true, child_depth, &format!("{path}.onTrue"))?;
    let on_false = ingest_steps_inner(b.on_false, child_depth, &format!("{path}.onFalse"))?;
    Ok(PipelineBranchSpec {
        predicate: b.predicate,
        on_true,
        on_false,
    })
}

fn ingest_step(
    f: PipelineStepFile,
    depth: usize,
    path: &str,
) -> Result<PipelineStepSpec, BlueprintError> {
    let PipelineStepFile {
        action,
        id,
        branch,
        parallel,
    } = f;
    let arms_nonempty = parallel.as_ref().is_some_and(|p| !p.is_empty());
    let has_branch = branch.is_some();
    if arms_nonempty && has_branch {
        return Err(BlueprintError::BranchAndParallel(path.to_string()));
    }
    if arms_nonempty {
        let action_nonempty = action
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .is_some();
        if action_nonempty {
            return Err(BlueprintError::BranchWithAction(path.to_string()));
        }
        if depth + 1 > MAX_PIPELINE_CONTROL_FLOW_NEST_DEPTH {
            return Err(BlueprintError::MaxNestingDepth(
                path.to_string(),
                MAX_PIPELINE_CONTROL_FLOW_NEST_DEPTH,
            ));
        }
        let parallel_files = parallel.expect("arms_nonempty implies Some");
        let arms: Vec<Vec<PipelineStepSpec>> = parallel_files
            .into_iter()
            .enumerate()
            .map(|(ai, arm_files)| {
                ingest_steps_inner(arm_files, depth + 1, &format!("{path}.parallel[{ai}]"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        for (ai, arm) in arms.iter().enumerate() {
            validate_parallel_arm_readonly(arm, &format!("{path}.parallel[{ai}]"))?;
        }
        return Ok(PipelineStepSpec {
            id: id.map(|x| x.trim().to_string()).filter(|x| !x.is_empty()),
            action: None,
            branch: None,
            parallel: Some(arms),
        });
    }
    if let Some(bf) = branch {
        let action_nonempty = action
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .is_some();
        if action_nonempty {
            return Err(BlueprintError::BranchWithAction(path.to_string()));
        }
        if depth + 1 > MAX_PIPELINE_CONTROL_FLOW_NEST_DEPTH {
            return Err(BlueprintError::MaxNestingDepth(
                path.to_string(),
                MAX_PIPELINE_CONTROL_FLOW_NEST_DEPTH,
            ));
        }
        let branch_spec = ingest_branch(bf, depth + 1, path)?;
        return Ok(PipelineStepSpec {
            id: id.map(|x| x.trim().to_string()).filter(|x| !x.is_empty()),
            action: None,
            branch: Some(branch_spec),
            parallel: None,
        });
    }

    let action = action.as_deref().map(str::trim).filter(|s| !s.is_empty());
    let Some(action) = action.map(|s| s.to_string()) else {
        return Err(BlueprintError::LinearRequiresAction(path.to_string()));
    };
    if !action_allowed_for_blueprint(action.as_str()) {
        return Err(BlueprintError::UnknownOrDisallowedAction(
            path.to_string(),
            action,
        ));
    }
    Ok(PipelineStepSpec {
        id: id.map(|x| x.trim().to_string()).filter(|x| !x.is_empty()),
        action: Some(action),
        branch: None,
        parallel: None,
    })
}

fn ingest_steps_inner(
    files: Vec<PipelineStepFile>,
    depth: usize,
    prefix: &str,
) -> Result<Vec<PipelineStepSpec>, BlueprintError> {
    let mut out = Vec::with_capacity(files.len());
    for (idx, sf) in files.into_iter().enumerate() {
        let p = step_path(prefix, idx);
        out.push(ingest_step(sf, depth, &p)?);
    }
    Ok(out)
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
    if file.steps.len() > MAX_PIPELINE_ROOT_STEPS {
        return Err(BlueprintError::TooManyRootSteps(
            file.steps.len(),
            MAX_PIPELINE_ROOT_STEPS,
        ));
    }
    let on_failure = parse_on_failure(file.on_failure.as_deref())?;
    let steps = ingest_steps_inner(file.steps, 0, "")?;
    let mut seen_ids = HashMap::new();
    validate_unique_step_ids(&steps, "", &mut seen_ids)?;
    validate_whitelisted_actions_tree(&steps, "")?;
    let nodes = count_nodes(&steps);
    if nodes > MAX_PIPELINE_TREE_NODES {
        return Err(BlueprintError::TooManyNodes(nodes));
    }
    Ok(PipelineBlueprint {
        schema_version: sv.to_string(),
        name,
        on_failure,
        steps,
    })
}

/// 从字节解析并校验（测试与工具复用）。
///
/// 校验包括：schema 版本、根步数/树节点数、控制流嵌套深度（[`MAX_PIPELINE_CONTROL_FLOW_NEST_DEPTH`]）、
/// `PARALLEL` 只读约束、非空 `id` 全树唯一、原子白名单与 `ingest` 一致性等。
pub fn parse_and_validate_blueprint_bytes(
    bytes: &[u8],
) -> Result<PipelineBlueprint, BlueprintError> {
    let file: PipelineBlueprintFile = serde_json::from_slice(bytes)?;
    validate_file(file)
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
        assert!(matches!(e, BlueprintError::UnknownOrDisallowedAction(_, _)));
    }

    #[test]
    fn rejects_bad_on_failure() {
        let j = br#"{"schemaVersion":"1.0","name":"x","onFailure":"PANIC","steps":[{"action":"init_turn"}]}"#;
        let e = parse_and_validate_blueprint_bytes(j).unwrap_err();
        assert!(matches!(e, BlueprintError::InvalidOnFailure(_)));
    }

    #[test]
    fn parses_branch_blueprint() {
        let j = r#"{
            "schemaVersion":"1.0",
            "name":"br",
            "steps":[
                {"action":"init_turn"},
                {"id":"b","branch":{
                    "predicate":{"type":"sceneIdEquals","sceneId":"x"},
                    "onTrue":[{"action":"init_turn"}],
                    "onFalse":[{"action":"init_turn"}]
                }}
            ]
        }"#;
        let bp = parse_and_validate_blueprint_bytes(j.as_bytes()).expect("parse");
        assert!(bp.steps[1].branch.is_some());
    }

    #[test]
    fn parses_parallel_readonly_blueprint() {
        let j = r#"{
            "schemaVersion":"1.0",
            "name":"p",
            "steps":[{"parallel":[
                [{"action":"memory_retrieve_short_term"}],
                [{"action":"memory_retrieve_long_term"}]
            ]}]
        }"#;
        let bp = parse_and_validate_blueprint_bytes(j.as_bytes()).expect("parse");
        assert!(bp.steps[0].parallel.as_ref().is_some_and(|a| a.len() == 2));
    }

    #[test]
    fn rejects_branch_inside_parallel_arm() {
        let j = r#"{
            "schemaVersion":"1.0",
            "name":"bad",
            "steps":[{"parallel":[[{
                "branch":{
                    "predicate":{"type":"sceneIdEquals","sceneId":"x"},
                    "onTrue":[{"action":"memory_retrieve_short_term"}],
                    "onFalse":[{"action":"memory_retrieve_long_term"}]
                }
            }]]}]
        }"#;
        let e = parse_and_validate_blueprint_bytes(j.as_bytes()).unwrap_err();
        assert!(matches!(e, BlueprintError::ParallelContainsBranch(_)));
    }

    #[test]
    fn parses_official_example_blueprints() {
        for name in [
            "simple_companion.ocblueprint",
            "minimal_chat.ocblueprint",
            "memory_heavy.ocblueprint",
            "deep_empathy.ocblueprint",
        ] {
            let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../examples/blueprints/")
                .join(name);
            let bytes = std::fs::read(&p).unwrap_or_else(|_| panic!("read {name}"));
            parse_and_validate_blueprint_bytes(&bytes).unwrap_or_else(|e| panic!("{name}: {e}"));
        }
    }

    #[test]
    fn rejects_parallel_with_write_action() {
        let j = r#"{
            "schemaVersion":"1.0",
            "name":"p",
            "steps":[{"parallel":[[{"action":"init_turn"}],[{"action":"memory_retrieve_long_term"}]]}]
        }"#;
        let e = parse_and_validate_blueprint_bytes(j.as_bytes()).unwrap_err();
        assert!(matches!(e, BlueprintError::ParallelContainsWrite(_, _)));
    }

    #[test]
    fn blueprint_errors_use_bracket_codes() {
        let e = parse_and_validate_blueprint_bytes(
            br#"{"schemaVersion":"1.0","name":"x","steps":[{"action":"nope"}]}"#,
        )
        .unwrap_err();
        let s = e.to_string();
        assert!(s.contains("[PIPELINE_ACTION_NOT_ALLOWED]"), "{s}");
    }

    #[test]
    fn parses_three_level_nested_branches() {
        let j = r#"{
            "schemaVersion":"1.0",
            "name":"n3",
            "steps":[{"branch":{
                "predicate":{"type":"sceneIdEquals","sceneId":"d"},
                "onTrue":[{"branch":{
                    "predicate":{"type":"sceneIdEquals","sceneId":"d"},
                    "onTrue":[{"branch":{
                        "predicate":{"type":"sceneIdEquals","sceneId":"d"},
                        "onTrue":[{"action":"init_turn"}],
                        "onFalse":[]
                    }}],
                    "onFalse":[]
                }}],
                "onFalse":[]
            }}]
        }"#;
        parse_and_validate_blueprint_bytes(j.as_bytes()).expect("three nested branches");
    }

    #[test]
    fn rejects_fourth_level_nested_branch() {
        let j = r#"{
            "schemaVersion":"1.0",
            "name":"n4",
            "steps":[{"branch":{
                "predicate":{"type":"sceneIdEquals","sceneId":"x"},
                "onTrue":[{"branch":{
                    "predicate":{"type":"sceneIdEquals","sceneId":"x"},
                    "onTrue":[{"branch":{
                        "predicate":{"type":"sceneIdEquals","sceneId":"x"},
                        "onTrue":[{"branch":{
                            "predicate":{"type":"sceneIdEquals","sceneId":"x"},
                            "onTrue":[{"action":"init_turn"}],
                            "onFalse":[]
                        }}],
                        "onFalse":[]
                    }}],
                    "onFalse":[]
                }}],
                "onFalse":[]
            }}]
        }"#;
        let e = parse_and_validate_blueprint_bytes(j.as_bytes()).unwrap_err();
        assert!(matches!(e, BlueprintError::MaxNestingDepth(_, _)));
        assert!(
            e.to_string().contains("[PIPELINE_MAX_NESTING_DEPTH]"),
            "{}",
            e
        );
    }

    #[test]
    fn rejects_duplicate_step_id_across_tree() {
        let j = r#"{
            "schemaVersion":"1.0",
            "name":"dup",
            "steps":[
                {"id":"x","action":"init_turn"},
                {"id":"x","action":"ensure_role_runtime"}
            ]
        }"#;
        let e = parse_and_validate_blueprint_bytes(j.as_bytes()).unwrap_err();
        assert!(matches!(e, BlueprintError::DuplicateStepId { .. }));
        assert!(
            e.to_string().contains("[PIPELINE_DUPLICATE_STEP_ID]"),
            "{}",
            e
        );
    }

    #[test]
    fn parse_json_error_is_pipeline_parse() {
        let e = parse_and_validate_blueprint_bytes(b"{not json").unwrap_err();
        assert!(matches!(e, BlueprintError::Json(_)));
        assert!(e.to_string().contains("[PIPELINE_PARSE_ERROR]"), "{}", e);
    }

    #[test]
    fn unsupported_schema_version_error_code() {
        let j = br#"{"schemaVersion":"9.0","name":"x","steps":[{"action":"init_turn"}]}"#;
        let e = parse_and_validate_blueprint_bytes(j).unwrap_err();
        assert!(matches!(e, BlueprintError::UnsupportedSchemaVersion(_, _)));
        assert!(e.to_string().contains("[PIPELINE_SCHEMA_VERSION]"), "{}", e);
    }

    #[test]
    fn rejects_branch_and_parallel_on_same_step() {
        let j = r#"{
            "schemaVersion":"1.0",
            "name":"mix",
            "steps":[{
                "id":"m",
                "branch":{
                    "predicate":{"type":"sceneIdEquals","sceneId":"x"},
                    "onTrue":[],
                    "onFalse":[]
                },
                "parallel":[[{"action":"memory_retrieve_short_term"}]]
            }]
        }"#;
        let e = parse_and_validate_blueprint_bytes(j.as_bytes()).unwrap_err();
        assert!(matches!(e, BlueprintError::BranchAndParallel(_)));
    }
}

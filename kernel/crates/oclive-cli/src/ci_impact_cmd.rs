//! Domain-aware CI impact planning and explanation.

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{bail, Context, Result};
use clap::{Parser, ValueEnum};
use oclive_ci_plan::{CiPlan, GateStrength, PlanRequest, Planner, ValidationTier};

const DEFAULT_IMPACT_MAP: &str = "data/ci/impact-map.v1.json";
const DEFAULT_CATALOG: &str = "data/ci/validation-catalog.v1.json";
const DEFAULT_PLAN_OUTPUT: &str = "target/oclive-ci/plan.json";

#[derive(Parser, Debug)]
pub struct CiPlanArgs {
    /// Repository root.
    #[arg(short = 'o', long = "path", default_value = ".")]
    pub path: PathBuf,
    /// Git base revision used when changed paths are not supplied explicitly.
    #[arg(long, default_value = "HEAD^")]
    pub base: String,
    /// Git head revision used when changed paths are not supplied explicitly.
    #[arg(long, default_value = "HEAD")]
    pub head: String,
    /// Validation policy from the trusted catalog.
    #[arg(long, default_value = "pull_request")]
    pub policy: String,
    /// Central impact map path, relative to the repository root unless absolute.
    #[arg(long, default_value = DEFAULT_IMPACT_MAP)]
    pub impact_map: PathBuf,
    /// Trusted validation catalog path, relative to the repository root unless absolute.
    #[arg(long, default_value = DEFAULT_CATALOG)]
    pub catalog: PathBuf,
    /// Explicit changed path; repeat to avoid invoking Git.
    #[arg(long = "changed-file")]
    pub changed_files: Vec<String>,
    /// Read newline-delimited changed paths from a file.
    #[arg(long = "changed-files-from")]
    pub changed_files_from: Option<PathBuf>,
    /// Mark the plan as observational; consumers must not skip jobs from a shadow plan.
    #[arg(long)]
    pub shadow: bool,
    /// JSON output path, relative to the repository root unless absolute.
    #[arg(long, default_value = DEFAULT_PLAN_OUTPUT)]
    pub output: PathBuf,
}

#[derive(Parser, Debug)]
pub struct CiExplainArgs {
    /// Repository root used to resolve relative paths.
    #[arg(short = 'o', long = "path", default_value = ".")]
    pub path: PathBuf,
    /// Existing plan JSON.
    #[arg(long, default_value = DEFAULT_PLAN_OUTPUT)]
    pub plan: PathBuf,
    /// Human-readable output format.
    #[arg(long, value_enum, default_value_t = ExplainFormat::Text)]
    pub format: ExplainFormat,
    /// Optional output file; stdout is used when omitted.
    #[arg(long)]
    pub output: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ExplainFormat {
    Text,
    Markdown,
}

pub fn run_plan(args: CiPlanArgs) -> Result<()> {
    let root = canonical_root(&args.path)?;
    let planner = Planner::load(&root, &args.impact_map, &args.catalog)
        .context("load OCLive CI planning contracts")?;
    let (base_sha, head_sha, changed_files) = resolve_plan_input(&root, &args)?;
    let plan = planner
        .plan(PlanRequest {
            base_sha,
            head_sha,
            policy: args.policy,
            shadow: args.shadow,
            changed_files,
        })
        .context("compute OCLive CI impact plan")?;
    let output = resolve_path(&root, &args.output);
    write_plan(&output, &plan)?;
    print!("{}", render_text(&plan));
    println!("Plan JSON: {}", output.display());
    Ok(())
}

pub fn run_explain(args: CiExplainArgs) -> Result<()> {
    let root = canonical_root(&args.path)?;
    let input = resolve_path(&root, &args.plan);
    let plan: CiPlan = serde_json::from_slice(
        &fs::read(&input).with_context(|| format!("read plan {}", input.display()))?,
    )
    .with_context(|| format!("parse plan {}", input.display()))?;
    let rendered = match args.format {
        ExplainFormat::Text => render_text(&plan),
        ExplainFormat::Markdown => render_markdown(&plan),
    };
    if let Some(output) = args.output {
        let output = resolve_path(&root, &output);
        write_text(&output, &rendered)?;
        println!("Explanation: {}", output.display());
    } else {
        print!("{rendered}");
    }
    Ok(())
}

fn resolve_plan_input(root: &Path, args: &CiPlanArgs) -> Result<(String, String, Vec<String>)> {
    let mut changed_files = args.changed_files.clone();
    if let Some(input) = &args.changed_files_from {
        let input = resolve_path(root, input);
        let contents = fs::read_to_string(&input)
            .with_context(|| format!("read changed paths from {}", input.display()))?;
        changed_files.extend(
            contents
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(ToOwned::to_owned),
        );
    }

    let base_sha = git_rev_parse(root, &args.base)?;
    let head_sha = git_rev_parse(root, &args.head)?;
    if changed_files.is_empty() {
        changed_files = git_changed_files(root, &base_sha, &head_sha)?;
    }
    Ok((base_sha, head_sha, changed_files))
}

fn git_rev_parse(root: &Path, revision: &str) -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--verify", revision])
        .current_dir(root)
        .output()
        .with_context(|| format!("run git rev-parse for `{revision}`"))?;
    if !output.status.success() {
        bail!(
            "git rev-parse `{revision}` failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn git_changed_files(root: &Path, base_sha: &str, head_sha: &str) -> Result<Vec<String>> {
    let output = Command::new("git")
        .args([
            "diff",
            "--name-only",
            "--diff-filter=ACMRD",
            "-z",
            base_sha,
            head_sha,
            "--",
        ])
        .current_dir(root)
        .output()
        .context("run git diff for CI impact planning")?;
    if !output.status.success() {
        bail!(
            "git diff failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|value| !value.is_empty())
        .map(|value| {
            String::from_utf8(value.to_vec())
                .context("git returned a changed path that is not valid UTF-8")
        })
        .collect()
}

fn write_plan(path: &Path, plan: &CiPlan) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(plan).context("serialize CI plan")?;
    bytes.push(b'\n');
    write_bytes(path, &bytes)
}

fn write_text(path: &Path, value: &str) -> Result<()> {
    write_bytes(path, value.as_bytes())
}

fn write_bytes(path: &Path, value: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create output directory {}", parent.display()))?;
    }
    fs::write(path, value).with_context(|| format!("write {}", path.display()))
}

fn canonical_root(path: &Path) -> Result<PathBuf> {
    path.canonicalize()
        .with_context(|| format!("resolve repository root {}", path.display()))
}

fn resolve_path(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_owned()
    } else {
        root.join(path)
    }
}

fn render_text(plan: &CiPlan) -> String {
    let mode = if plan.shadow { "shadow" } else { "active" };
    let fallback = if plan.fallback.full {
        "full"
    } else {
        "targeted"
    };
    let mut output = format!(
        "OCLive CI impact plan ({mode}, {fallback})\nBase: {}\nHead: {}\nPolicy: {}\nChanged paths: {}\nDirect modules: {}\nAffected modules: {}\nSelected validators: {}\n",
        plan.base_sha,
        plan.head_sha,
        plan.policy,
        plan.changed_files.len(),
        join_ids(&plan.direct_modules),
        join_ids(&plan.affected_modules),
        plan.selected_validators
            .iter()
            .map(|value| value.id.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
    if !plan.fallback.reasons.is_empty() {
        output.push_str("Fallback reasons:\n");
        for reason in &plan.fallback.reasons {
            output.push_str(&format!("  - {reason}\n"));
        }
    }
    for warning in &plan.warnings {
        output.push_str(&format!("Warning: {warning}\n"));
    }
    output
}

fn render_markdown(plan: &CiPlan) -> String {
    let mode = if plan.shadow { "Shadow" } else { "Active" };
    let scope = if plan.fallback.full {
        "Full fallback"
    } else {
        "Targeted"
    };
    let mut output = format!(
        "## OCLive CI impact plan\n\n- Mode: **{mode}**\n- Scope: **{scope}**\n- Policy: `{}`\n- Base/head: `{}` → `{}`\n- Changed paths: **{}**\n- Direct modules: {}\n- Affected modules: {}\n\n### Selected validators\n\n",
        markdown_code(&plan.policy),
        markdown_code(&plan.base_sha),
        markdown_code(&plan.head_sha),
        plan.changed_files.len(),
        markdown_ids(&plan.direct_modules),
        markdown_ids(&plan.affected_modules)
    );
    if plan.selected_validators.is_empty() {
        output.push_str("_None._\n");
    } else {
        for validator in &plan.selected_validators {
            output.push_str(&format!(
                "- `{}` — `{}` / `{}` — {}\n",
                markdown_code(&validator.id),
                tier_label(validator.tier),
                gate_label(validator.gate),
                validator
                    .reasons
                    .iter()
                    .map(|reason| format!("`{}`", markdown_code(reason)))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }
    if !plan.fallback.reasons.is_empty() {
        output.push_str("\n### Full-fallback reasons\n\n");
        for reason in &plan.fallback.reasons {
            output.push_str(&format!("- `{}`\n", markdown_code(reason)));
        }
    }
    if !plan.warnings.is_empty() {
        output.push_str("\n### Warnings\n\n");
        for warning in &plan.warnings {
            output.push_str(&format!("- {}\n", markdown_code(warning)));
        }
    }
    output
}

fn join_ids(values: &[oclive_ci_plan::ReasonedSelection]) -> String {
    if values.is_empty() {
        "(none)".to_owned()
    } else {
        values
            .iter()
            .map(|value| value.id.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn markdown_ids(values: &[oclive_ci_plan::ReasonedSelection]) -> String {
    if values.is_empty() {
        "_none_".to_owned()
    } else {
        values
            .iter()
            .map(|value| format!("`{}`", markdown_code(&value.id)))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn markdown_code(value: &str) -> String {
    value.replace('`', "\\`").replace(['\r', '\n'], " ")
}

fn tier_label(tier: ValidationTier) -> &'static str {
    match tier {
        ValidationTier::Fast => "fast",
        ValidationTier::Pr => "pr",
        ValidationTier::Merge => "merge",
        ValidationTier::Nightly => "nightly",
        ValidationTier::Release => "release",
    }
}

fn gate_label(gate: GateStrength) -> &'static str {
    match gate {
        GateStrength::Required => "required",
        GateStrength::Advisory => "advisory",
        GateStrength::Quarantined => "quarantined",
    }
}

#[cfg(test)]
mod tests {
    use oclive_ci_plan::{FallbackDecision, ReasonedSelection, PLAN_SCHEMA_VERSION};

    use super::*;

    fn sample_plan() -> CiPlan {
        CiPlan {
            schema_version: PLAN_SCHEMA_VERSION,
            base_sha: "abc".to_owned(),
            head_sha: "def".to_owned(),
            policy: "pull_request".to_owned(),
            shadow: true,
            changed_files: vec!["kernel/src/lib.rs".to_owned()],
            direct_modules: vec![ReasonedSelection {
                id: "oclive.kernel".to_owned(),
                reasons: vec!["changed_path".to_owned()],
            }],
            affected_modules: vec![ReasonedSelection {
                id: "oclive.kernel".to_owned(),
                reasons: vec!["direct_change".to_owned()],
            }],
            selected_profiles: Vec::new(),
            selected_validators: Vec::new(),
            skipped_validators: Vec::new(),
            fallback: FallbackDecision {
                full: false,
                reasons: Vec::new(),
            },
            warnings: Vec::new(),
            impact_map_sha256: "0".repeat(64),
            validation_catalog_sha256: "1".repeat(64),
        }
    }

    #[test]
    fn markdown_explanation_is_summary_safe() {
        let mut plan = sample_plan();
        plan.warnings.push("line one\nline two `quoted`".to_owned());
        let rendered = render_markdown(&plan);
        assert!(rendered.contains("Mode: **Shadow**"));
        assert!(rendered.contains("`oclive.kernel`"));
        assert!(!rendered.contains("line one\nline two"));
    }

    #[test]
    fn text_explanation_names_scope_and_modules() {
        let rendered = render_text(&sample_plan());
        assert!(rendered.contains("shadow, targeted"));
        assert!(rendered.contains("Direct modules: oclive.kernel"));
    }

    #[test]
    fn enum_labels_match_contract_json() {
        assert_eq!(tier_label(ValidationTier::Pr), "pr");
        assert_eq!(gate_label(GateStrength::Required), "required");
    }
}

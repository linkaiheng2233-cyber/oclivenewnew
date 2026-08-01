//! `lint --audit-ci` — verify required cargo-audit ownership in GitHub Actions CI.

use anyhow::{bail, Context, Result};
use serde_yaml_ng::{Mapping, Value};
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct AuditCiStatus {
    pub owners: Vec<String>,
    pub soft_owners: Vec<String>,
}

pub fn run_audit_ci(start: &Path) -> Result<()> {
    let ci = find_ci_yml(start)?;
    let text = std::fs::read_to_string(&ci)?;
    println!("oclive lint --audit-ci");
    println!("  workflow: {}", ci.display());

    let status = inspect_audit_ci(&text)?;

    if status.owners.is_empty() {
        println!("  [FAIL] No cargo audit owner found in ci.yml");
        println!(
            "  Suggestion: add a required `cargo audit` step or the OCLive Dimension 5 acceptance gate"
        );
        bail!("cargo audit owner missing from CI workflow");
    }
    println!(
        "  [PASS] cargo audit owner(s): {}",
        status.owners.join(", ")
    );

    if status.soft_owners.is_empty() {
        println!("  [PASS] cargo audit owners block CI on failure");
    } else {
        println!(
            "  [WARN] cargo audit owner(s) allow failure: {}",
            status.soft_owners.join(", ")
        );
        println!(
            "  Suggestion: set continue-on-error: false after dependency upgrades so high-severity advisories block merges"
        );
    }
    Ok(())
}

pub(crate) fn inspect_audit_ci(text: &str) -> Result<AuditCiStatus> {
    let document: Value = serde_yaml_ng::from_str(text).context("parse GitHub Actions workflow")?;
    let Some(jobs) = mapping_value(document.as_mapping(), "jobs").and_then(Value::as_mapping)
    else {
        bail!("GitHub Actions workflow has no jobs mapping");
    };

    let mut owners = Vec::new();
    let mut soft_owners = Vec::new();
    for (job_id, job_value) in jobs {
        let (Some(job_id), Some(job)) = (job_id.as_str(), job_value.as_mapping()) else {
            continue;
        };
        let job_allows_failure =
            continue_on_error_enabled(mapping_value(Some(job), "continue-on-error"));
        let Some(steps) = mapping_value(Some(job), "steps").and_then(Value::as_sequence) else {
            continue;
        };

        let mut owns_audit = false;
        let mut audit_step_allows_failure = false;
        for step in steps {
            let Some(step) = step.as_mapping() else {
                continue;
            };
            let Some(run) = mapping_value(Some(step), "run").and_then(Value::as_str) else {
                continue;
            };
            if is_audit_owner_run(run) {
                owns_audit = true;
                audit_step_allows_failure |=
                    continue_on_error_enabled(mapping_value(Some(step), "continue-on-error"));
            }
        }

        if owns_audit {
            owners.push(job_id.to_owned());
            if job_allows_failure || audit_step_allows_failure {
                soft_owners.push(job_id.to_owned());
            }
        }
    }
    owners.sort();
    soft_owners.sort();
    Ok(AuditCiStatus {
        owners,
        soft_owners,
    })
}

fn mapping_value<'a>(mapping: Option<&'a Mapping>, key: &str) -> Option<&'a Value> {
    mapping?.get(Value::String(key.to_owned()))
}

fn continue_on_error_enabled(value: Option<&Value>) -> bool {
    match value {
        None | Some(Value::Bool(false)) | Some(Value::Null) => false,
        Some(_) => true,
    }
}

fn is_audit_owner_run(run: &str) -> bool {
    run.lines().map(str::trim).any(|line| {
        let command = line.trim_start_matches('-').trim();
        (command.starts_with("cargo audit")
            && command
                .chars()
                .nth("cargo audit".len())
                .is_none_or(char::is_whitespace))
            || (command.contains("dimension5-acceptance.mjs") && command.contains("--ci"))
    })
}

fn find_ci_yml(start: &Path) -> Result<PathBuf> {
    let mut dir = start.canonicalize().unwrap_or_else(|_| start.to_path_buf());
    for _ in 0..12 {
        let cand = dir.join(".github/workflows/ci.yml");
        if cand.is_file() {
            return Ok(cand);
        }
        if !dir.pop() {
            break;
        }
    }
    bail!(
        "could not find .github/workflows/ci.yml from {}",
        start.display()
    );
}

#[cfg(test)]
mod tests {
    use super::inspect_audit_ci;

    #[test]
    fn accepts_dimension5_owner_despite_unrelated_soft_jobs() {
        let workflow = r#"
jobs:
  dimension5-acceptance:
    steps:
      - run: node scripts/dimension5-acceptance.mjs --ci
  loom:
    continue-on-error: true
    steps:
      - run: cargo test --test loom_concurrency
"#;
        let status = inspect_audit_ci(workflow).expect("workflow should parse");
        assert_eq!(status.owners, ["dimension5-acceptance"]);
        assert!(status.soft_owners.is_empty());
    }

    #[test]
    fn scopes_continue_on_error_to_the_audit_owner() {
        let workflow = r#"
jobs:
  cargo-audit:
    continue-on-error: true
    steps:
      - run: cargo install cargo-audit --version 0.22.1 --locked
      - run: cargo audit
"#;
        let status = inspect_audit_ci(workflow).expect("workflow should parse");
        assert_eq!(status.owners, ["cargo-audit"]);
        assert_eq!(status.soft_owners, ["cargo-audit"]);
    }

    #[test]
    fn install_command_does_not_count_as_an_audit_owner() {
        let workflow = r#"
jobs:
  setup:
    steps:
      - run: cargo install cargo-audit --version 0.22.1 --locked
"#;
        let status = inspect_audit_ci(workflow).expect("workflow should parse");
        assert!(status.owners.is_empty());
    }
}

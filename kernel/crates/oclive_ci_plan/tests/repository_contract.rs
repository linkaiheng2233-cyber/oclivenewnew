use std::{fs, path::PathBuf};

use oclive_ci_plan::{GateStrength, PlanRequest, Planner};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .and_then(|path| path.parent())
        .expect("crate must stay under kernel/crates")
        .to_owned()
}

fn request(policy: &str, path: &str) -> PlanRequest {
    PlanRequest {
        base_sha: "base".to_owned(),
        head_sha: "head".to_owned(),
        policy: policy.to_owned(),
        shadow: true,
        changed_files: vec![path.to_owned()],
    }
}

#[test]
fn repository_catalog_maps_every_declared_workflow_job() {
    let root = repo_root();
    let planner = Planner::load(
        &root,
        "data/ci/impact-map.v1.json",
        "data/ci/validation-catalog.v1.json",
    )
    .expect("load repository contracts");
    let plan = planner
        .plan(request("release", "unmapped/fail-safe.txt"))
        .expect("release fallback plan");
    assert!(plan.fallback.full);
    assert_eq!(plan.selected_validators.len(), 19);

    let workflow =
        fs::read_to_string(root.join(".github/workflows/ci.yml")).expect("read main workflow");
    for validator in &plan.selected_validators {
        for job in &validator.workflow_jobs {
            assert!(
                workflow.contains(&format!("\n  {job}:")),
                "validator `{}` references missing workflow job `{job}`",
                validator.id
            );
        }
    }
}

#[test]
fn repository_workflow_keeps_expensive_validation_ownership_disjoint() {
    let root = repo_root();
    let planner = Planner::load(
        &root,
        "data/ci/impact-map.v1.json",
        "data/ci/validation-catalog.v1.json",
    )
    .expect("load repository contracts");
    let plan = planner
        .plan(request("release", "unmapped/fail-safe.txt"))
        .expect("release fallback plan");

    let cargo_audit = plan
        .selected_validators
        .iter()
        .find(|validator| validator.id == "cargo-audit")
        .expect("cargo-audit validator");
    assert_eq!(cargo_audit.workflow_jobs, ["dimension5-acceptance"]);

    let npm_audit = plan
        .selected_validators
        .iter()
        .find(|validator| validator.id == "npm-audit")
        .expect("npm-audit validator");
    assert_eq!(npm_audit.gate, GateStrength::Required);

    let workflow =
        fs::read_to_string(root.join(".github/workflows/ci.yml")).expect("read main workflow");
    assert!(workflow.contains(
        "cargo clippy --locked --workspace --exclude oclive-cli --all-targets --all-features"
    ));
    assert!(workflow.contains("cargo test --locked --workspace --exclude oclive-cli"));
    assert!(workflow.contains("cargo test --locked -p oclive-cli -- --test-threads=1"));
    assert!(workflow.contains("npm run typecheck"));
    assert!(!workflow.contains("\n  cargo-audit:"));
}

#[test]
fn repository_workflows_share_the_node_runtime_baseline() {
    let root = repo_root();
    let nvmrc = fs::read_to_string(root.join(".nvmrc")).expect("read .nvmrc");
    assert_eq!(nvmrc.trim(), "22");

    let package = fs::read_to_string(root.join("package.json")).expect("read package.json");
    assert!(package.contains(r#""node": ">=22""#));

    let workflows = root.join(".github/workflows");
    for entry in fs::read_dir(workflows).expect("read workflows") {
        let path = entry.expect("workflow entry").path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("yml") {
            continue;
        }
        let workflow = fs::read_to_string(&path).expect("read workflow");
        let setup_count = workflow.matches("actions/setup-node@").count();
        if setup_count == 0 {
            continue;
        }
        assert_eq!(
            workflow.matches("node-version-file: \".nvmrc\"").count(),
            setup_count,
            "{} must source every setup-node version from .nvmrc",
            path.display()
        );
        assert!(
            !workflow.contains("node-version:"),
            "{} must not duplicate the Node version literal",
            path.display()
        );
    }
}

#[test]
fn repository_rules_cover_targeted_and_fail_safe_examples() {
    let root = repo_root();
    let planner = Planner::load(
        &root,
        "data/ci/impact-map.v1.json",
        "data/ci/validation-catalog.v1.json",
    )
    .expect("load repository contracts");

    let docs = planner
        .plan(request(
            "pull_request",
            "creator-docs/roadmap/SOMEDAY_TOOLCHAIN_CI.md",
        ))
        .expect("docs plan");
    assert!(!docs.fallback.full);
    assert_eq!(docs.direct_modules[0].id, "oclive.docs");
    assert_eq!(
        docs.selected_validators
            .iter()
            .map(|validator| validator.id.as_str())
            .collect::<Vec<_>>(),
        vec!["dimension5-acceptance", "stale-paths"]
    );

    let plugin = planner
        .plan(request(
            "pull_request",
            "distros/chat-pro/plugins/com.oclive.voice.asr/manifest.json",
        ))
        .expect("plugin plan");
    assert!(!plugin.fallback.full);
    assert!(plugin
        .direct_modules
        .iter()
        .any(|module| module.id == "oclive.plugins"));
    assert!(plugin
        .affected_modules
        .iter()
        .any(|module| module.id == "oclive.chat-pro"));

    let workspace = planner
        .plan(request("pull_request", "Cargo.toml"))
        .expect("workspace plan");
    assert!(workspace.fallback.full);
    assert!(workspace
        .fallback
        .reasons
        .iter()
        .any(|reason| reason.starts_with("risk_override:workspace-dependencies:")));

    let scaffold = planner
        .plan(request(
            "pull_request",
            "kernel/crates/oclive_scaffold/src/validation.rs",
        ))
        .expect("scaffold contract plan");
    assert!(!scaffold.fallback.full);
    assert_eq!(scaffold.direct_modules[0].id, "oclive.scaffold");
    assert!(scaffold
        .affected_modules
        .iter()
        .any(|module| module.id == "oclive.cli"));
    for validator in ["rust-workspace", "dimension5-acceptance", "cli"] {
        assert!(
            scaffold
                .selected_validators
                .iter()
                .any(|selection| selection.id == validator),
            "missing scaffold validator {validator}"
        );
    }
}

use std::{fs, path::PathBuf};

use oclive_ci_plan::{PlanRequest, Planner};

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
}

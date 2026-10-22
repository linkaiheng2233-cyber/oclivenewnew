use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use oclive_ci_plan::{PlanRequest, Planner};

#[derive(Debug, serde::Deserialize)]
struct Scheme {
    id: String,
    status: String,
    includes: Vec<String>,
    jobs: Vec<String>,
    derived_from: DerivedFrom,
}

#[derive(Debug, serde::Deserialize)]
struct DerivedFrom {
    impact_map: String,
    catalog: String,
    planner: String,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .and_then(|path| path.parent())
        .expect("crate must stay under kernel/crates")
        .to_owned()
}

fn git_rev(root: &Path, spec: &str) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", spec])
        .output()
        .expect("run git rev-parse");
    assert!(output.status.success(), "git rev-parse {spec} failed");
    String::from_utf8(output.stdout).expect("utf8").trim().to_owned()
}

fn expand_include(root: &Path, pattern: &str) -> Vec<String> {
    if let Some(dir) = pattern.strip_suffix("/**") {
        let mut files = Vec::new();
        walk(root, dir, &mut files);
        files.sort();
        return files;
    }
    vec![pattern.to_owned()]
}

fn walk(root: &Path, dir: &str, files: &mut Vec<String>) {
    let base = root.join(dir);
    let Ok(entries) = fs::read_dir(&base) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let relative = path
            .strip_prefix(root)
            .expect("inside repo")
            .to_string_lossy()
            .replace('\\', "/");
        if path.is_dir() {
            walk(root, &relative, files);
        } else if path.is_file() {
            files.push(relative);
        }
    }
}

fn plan_files(root: &Path, files: Vec<String>) -> oclive_ci_plan::CiPlan {
    let planner = Planner::load(
        root,
        "data/ci/impact-map.v1.json",
        "data/ci/validation-catalog.v1.json",
    )
    .expect("load repository contracts");
    planner
        .plan(PlanRequest {
            base_sha: "base".to_owned(),
            head_sha: "head".to_owned(),
            policy: "pull_request".to_owned(),
            shadow: true,
            changed_files: files,
        })
        .expect("plan")
}

fn selected_jobs(plan: &oclive_ci_plan::CiPlan) -> Vec<String> {
    let mut jobs: Vec<String> = plan
        .selected_validators
        .iter()
        .map(|validator| validator.id.clone())
        .collect();
    jobs.sort();
    jobs
}

#[test]
fn every_scheme_is_fresh_and_matches_the_planner() {
    let root = repo_root();
    let schemes_dir = root.join("data/ci/schemes");
    let impact_map = git_rev(&root, "HEAD:data/ci/impact-map.v1.json");
    let catalog = git_rev(&root, "HEAD:data/ci/validation-catalog.v1.json");
    let planner = git_rev(&root, "HEAD:kernel/crates/oclive_ci_plan");

    let mut seen = 0;
    for entry in fs::read_dir(&schemes_dir).expect("read schemes") {
        let path = entry.expect("entry").path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let scheme: Scheme =
            serde_json::from_str(&fs::read_to_string(&path).expect("read scheme")).expect("parse scheme");
        seen += 1;
        assert!(
            matches!(scheme.status.as_str(), "draft" | "active"),
            "{}: invalid status",
            scheme.id
        );
        assert_eq!(scheme.derived_from.impact_map, impact_map, "{}: stale impact_map", scheme.id);
        assert_eq!(scheme.derived_from.catalog, catalog, "{}: stale catalog", scheme.id);
        assert_eq!(scheme.derived_from.planner, planner, "{}: stale planner", scheme.id);

        let mut files = Vec::new();
        for include in &scheme.includes {
            files.extend(expand_include(&root, include));
        }
        files.sort();
        files.dedup();
        assert!(!files.is_empty(), "{}: includes resolve to nothing", scheme.id);

        let plan = plan_files(&root, files);
        assert!(
            !plan.fallback.full,
            "{}: planner falls back to full: {:?}",
            scheme.id, plan.fallback.reasons
        );
        let mut expected = scheme.jobs.clone();
        expected.sort();
        assert_eq!(
            selected_jobs(&plan),
            expected,
            "{}: scheme jobs do not equal the planner output",
            scheme.id
        );
    }
    assert!(seen > 0, "no schemes found");
}

#[test]
fn docs_tooling_paths_are_scoped_and_unknown_scripts_stay_full() {
    let root = repo_root();

    let scoped = plan_files(&root, vec!["scripts/ai-docs.mjs".to_owned()]);
    assert!(!scoped.fallback.full);
    assert_eq!(scoped.direct_modules.len(), 1);
    assert_eq!(scoped.direct_modules[0].id, "oclive.docs-tooling");
    assert_eq!(
        selected_jobs(&scoped),
        vec![
            "dimension5-acceptance".to_owned(),
            "layering-ratchet".to_owned(),
            "stale-paths".to_owned()
        ]
    );

    let unknown = plan_files(&root, vec!["scripts/check-reference-host.mjs".to_owned()]);
    assert!(unknown.fallback.full);
    assert!(unknown
        .fallback
        .reasons
        .iter()
        .any(|reason| reason.starts_with("unmapped_changed_path")));
}

#[test]
fn catalog_paths_join_docs_and_docs_tooling() {
    let root = repo_root();
    let plan = plan_files(&root, vec!["handoff/catalog/records.json".to_owned()]);
    assert!(!plan.fallback.full);
    let mut modules: Vec<String> = plan.direct_modules.iter().map(|module| module.id.clone()).collect();
    modules.sort();
    assert_eq!(modules, vec!["oclive.docs".to_owned(), "oclive.docs-tooling".to_owned()]);
    assert_eq!(
        selected_jobs(&plan),
        vec![
            "dimension5-acceptance".to_owned(),
            "layering-ratchet".to_owned(),
            "stale-paths".to_owned()
        ]
    );
}

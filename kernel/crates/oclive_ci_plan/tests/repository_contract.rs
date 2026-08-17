use std::{collections::BTreeSet, fs, path::PathBuf};

use oclive_ci_plan::{GateStrength, PlanRequest, Planner, ValidationTier};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ShadowScenarioCorpus {
    schema_version: u32,
    evidence_kind: String,
    authoritative_ci_comparison: bool,
    scenarios: Vec<ShadowScenario>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ShadowScenario {
    id: String,
    policy: String,
    input_mode: Option<String>,
    changed_files: Vec<String>,
    expected: ShadowExpectation,
    review_note: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ShadowExpectation {
    direct_modules: Vec<String>,
    affected_modules: Vec<String>,
    selected_validators: Vec<String>,
    workflow_jobs: Vec<String>,
    fallback_full: bool,
    fallback_reason_prefixes: Vec<String>,
}

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

fn workflow_job_block<'a>(workflow: &'a str, job: &str) -> &'a str {
    let marker = format!("\n  {job}:");
    let start = workflow
        .find(&marker)
        .unwrap_or_else(|| panic!("missing workflow job `{job}`"));
    let tail = &workflow[start + marker.len()..];
    let end = tail
        .match_indices("\n  ")
        .find(|(offset, _)| {
            tail.as_bytes()
                .get(offset + 3)
                .is_some_and(|byte| !byte.is_ascii_whitespace())
        })
        .map(|(offset, _)| start + marker.len() + offset)
        .unwrap_or(workflow.len());
    &workflow[start..end]
}

#[test]
fn repository_catalog_maps_every_validator_to_its_execution_lane() {
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

    let main_workflow =
        fs::read_to_string(root.join(".github/workflows/ci.yml")).expect("read main workflow");
    let nightly_workflow = fs::read_to_string(root.join(".github/workflows/nightly-advisory.yml"))
        .expect("read nightly workflow");
    let flake_workflow = fs::read_to_string(root.join(".github/workflows/ci-rerun-flake.yml"))
        .expect("read CI flake workflow");
    for validator in &plan.selected_validators {
        let (workflow, lane) = if validator.tier == ValidationTier::Nightly {
            (&nightly_workflow, "nightly")
        } else {
            (&main_workflow, "main")
        };
        for job in &validator.workflow_jobs {
            assert!(
                workflow.contains(&format!("\n  {job}:")),
                "validator `{}` references missing {lane} workflow job `{job}`",
                validator.id,
            );
        }
    }

    for job in [
        "loom",
        "fuzz",
        "cli-bench",
        "visual-presentation-smoke",
        "e2e-tauri",
    ] {
        assert!(
            !main_workflow.contains(&format!("\n  {job}:")),
            "nightly job `{job}` must not run in the merge-gating workflow",
        );
    }
    assert_eq!(
        main_workflow.matches("    continue-on-error: true").count(),
        0,
        "the active planner and every main CI responsibility must fail closed",
    );
    let planner_block = workflow_job_block(&main_workflow, "ci-impact-plan");
    assert!(planner_block.contains("run_full: ${{ steps.execution.outputs.run_full }}"));
    assert!(planner_block.contains("selected_jobs: ${{ steps.execution.outputs.selected_jobs }}"));
    assert!(planner_block.contains("trusted_sha: ${{ steps.comparison.outputs.base }}"));
    assert!(planner_block.contains("Checkout trusted CI control plane"));
    assert!(planner_block.contains("--manifest-path \"$TRUSTED_ROOT/Cargo.toml\""));
    assert!(planner_block.contains("--changed-files-from"));
    assert!(planner_block.contains("--diff-filter=ACMRD -z"));
    assert!(planner_block.contains("changed-files.zlist"));
    assert!(planner_block.contains("--pr-draft \"$PR_DRAFT\""));
    assert!(planner_block.contains("--force-full-reason trusted_policy_bootstrap"));
    assert!(!planner_block.contains("--shadow"));
    assert!(main_workflow
        .contains("types: [opened, synchronize, reopened, ready_for_review, converted_to_draft]"));

    let main_jobs = plan
        .selected_validators
        .iter()
        .filter(|validator| validator.tier != ValidationTier::Nightly)
        .flat_map(|validator| validator.workflow_jobs.iter().cloned())
        .collect::<BTreeSet<_>>();
    for job in &main_jobs {
        let block = workflow_job_block(&main_workflow, job);
        assert!(
            block.contains("needs: ci-impact-plan"),
            "main workflow job `{job}` must depend on the impact planner"
        );
        assert!(
            block.contains("needs.ci-impact-plan.outputs.run_full == 'true'"),
            "main workflow job `{job}` must honor full fallback"
        );
        assert!(
            block.contains(&format!("'\"{job}\"'")),
            "main workflow job `{job}` must honor its selected_jobs coordinate"
        );
    }

    let gate_block = workflow_job_block(&main_workflow, "ci-gate");
    assert!(gate_block.contains("'ci-draft-gate' || 'ci-gate'"));
    assert!(gate_block.contains("if: ${{ always() }}"));
    assert!(gate_block.contains("ref: ${{ needs.ci-impact-plan.outputs.trusted_sha }}"));
    assert!(gate_block.contains("target/oclive-ci/trusted/scripts/ci-execution-policy.mjs"));
    assert!(gate_block.contains("node \"$POLICY_SCRIPT\" verify --needs-env NEEDS_JSON"));
    assert!(gate_block.contains("actions/download-artifact@v7"));
    assert!(gate_block.contains("collect-ci-compare-evidence.mjs"));
    assert!(gate_block.contains("oclive-ci-compare-${{ github.run_id }}-${{ github.run_attempt }}"));
    assert!(gate_block.contains("retention-days: 90"));
    for job in &main_jobs {
        assert!(
            gate_block.contains(&format!("      - {job}\n")),
            "ci-gate must aggregate `{job}`"
        );
    }
    assert!(
        !nightly_workflow.contains("    continue-on-error: true"),
        "nightly failures must remain visible in their own workflow",
    );
    assert!(
        nightly_workflow.contains("--release --features loom-tests --test loom_concurrency"),
        "nightly Loom must execute the model tests, not only compile the disabled fixture",
    );
    assert!(!nightly_workflow.contains("loom_tests_require_cfg_loom"));
    assert!(flake_workflow.contains("workflow_dispatch:"));
    assert!(flake_workflow.contains("run_id:"));
    assert!(!flake_workflow.contains("workflow_run:"));
    assert!(flake_workflow.contains("--repo \"$GITHUB_REPOSITORY\""));
    assert!(flake_workflow.contains("              ci-gate)"));
    assert!(flake_workflow.contains("/tmp/oclive-ci-rust-failure.txt"));
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
    assert_eq!(npm_audit.command_id, "audit-npm");

    let workflow =
        fs::read_to_string(root.join(".github/workflows/ci.yml")).expect("read main workflow");
    assert!(workflow.contains(
        "cargo clippy --locked --workspace --exclude oclive-cli --all-targets --all-features"
    ));
    assert!(workflow.contains("cargo test --locked --workspace --exclude oclive-cli"));
    assert!(workflow.contains("cargo test --locked -p oclive-cli -- --test-threads=1"));
    assert!(workflow.contains("npm run typecheck"));
    assert!(workflow.contains("npm run audit:dependencies"));
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

    let execution_policy = planner
        .plan(request("pull_request", "scripts/ci-execution-policy.mjs"))
        .expect("execution policy plan");
    assert!(execution_policy.fallback.full);
    assert!(execution_policy
        .fallback
        .reasons
        .iter()
        .any(|reason| reason.starts_with("risk_override:ci-control-plane:")));

    let compare_collector = planner
        .plan(request(
            "pull_request",
            "scripts/collect-ci-compare-evidence.mjs",
        ))
        .expect("Compare evidence collector plan");
    assert!(compare_collector.fallback.full);
    assert!(compare_collector
        .fallback
        .reasons
        .iter()
        .any(|reason| reason.starts_with("risk_override:ci-control-plane:")));
}

#[test]
fn repository_shadow_scenario_corpus_matches_planner_contract() {
    let root = repo_root();
    let planner = Planner::load(
        &root,
        "data/ci/impact-map.v1.json",
        "data/ci/validation-catalog.v1.json",
    )
    .expect("load repository contracts");
    let corpus: ShadowScenarioCorpus = serde_json::from_str(
        &fs::read_to_string(root.join("data/ci/shadow-scenarios.v1.json"))
            .expect("read shadow scenario corpus"),
    )
    .expect("parse shadow scenario corpus");

    assert_eq!(corpus.schema_version, 1);
    assert_eq!(corpus.evidence_kind, "planner_contract_simulation");
    assert!(!corpus.authoritative_ci_comparison);
    assert!(corpus.scenarios.len() >= 20);

    for scenario in corpus.scenarios {
        assert!(!scenario.review_note.trim().is_empty(), "{}", scenario.id);
        assert!(
            matches!(
                scenario.input_mode.as_deref(),
                None | Some("arguments") | Some("nul_file")
            ),
            "{} uses an unsupported input mode",
            scenario.id
        );
        let expects_unmapped = scenario
            .expected
            .fallback_reason_prefixes
            .iter()
            .any(|prefix| prefix.starts_with("unmapped_changed_path:"));
        if !expects_unmapped {
            for changed_file in &scenario.changed_files {
                assert!(
                    root.join(changed_file).is_file(),
                    "{} references missing sample path `{changed_file}`",
                    scenario.id
                );
            }
        }
        let plan = planner
            .plan(PlanRequest {
                base_sha: "simulated-parent".to_owned(),
                head_sha: "simulated-head".to_owned(),
                policy: scenario.policy,
                shadow: true,
                changed_files: scenario.changed_files,
            })
            .unwrap_or_else(|error| panic!("{} failed to plan: {error}", scenario.id));
        let direct_modules = plan
            .direct_modules
            .iter()
            .map(|selection| selection.id.clone())
            .collect::<Vec<_>>();
        let affected_modules = plan
            .affected_modules
            .iter()
            .map(|selection| selection.id.clone())
            .collect::<Vec<_>>();
        let selected_validators = plan
            .selected_validators
            .iter()
            .map(|validator| validator.id.clone())
            .collect::<Vec<_>>();
        let workflow_jobs = plan
            .selected_validators
            .iter()
            .flat_map(|validator| validator.workflow_jobs.iter().cloned())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        assert_eq!(
            direct_modules, scenario.expected.direct_modules,
            "{}",
            scenario.id
        );
        assert_eq!(
            affected_modules, scenario.expected.affected_modules,
            "{}",
            scenario.id
        );
        assert_eq!(
            selected_validators, scenario.expected.selected_validators,
            "{}",
            scenario.id
        );
        assert_eq!(
            workflow_jobs, scenario.expected.workflow_jobs,
            "{}",
            scenario.id
        );
        assert_eq!(
            plan.fallback.full, scenario.expected.fallback_full,
            "{}",
            scenario.id
        );
        assert_eq!(
            plan.fallback.reasons.len(),
            scenario.expected.fallback_reason_prefixes.len(),
            "{}",
            scenario.id
        );
        for prefix in scenario.expected.fallback_reason_prefixes {
            assert!(
                plan.fallback
                    .reasons
                    .iter()
                    .any(|reason| reason.starts_with(&prefix)),
                "{} missing fallback reason prefix `{prefix}`",
                scenario.id
            );
        }
    }
}

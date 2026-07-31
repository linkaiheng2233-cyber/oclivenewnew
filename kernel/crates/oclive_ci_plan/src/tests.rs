use std::{fs, path::Path};

use serde_json::{json, Value};
use tempfile::TempDir;

use super::*;

struct Fixture {
    root: TempDir,
}

impl Fixture {
    fn new() -> Self {
        let root = tempfile::tempdir().expect("tempdir");
        fs::create_dir_all(root.path().join("data/ci/modules"))
            .expect("create fixture directories");
        Self { root }
    }

    fn write_json(&self, relative: &str, value: &Value) {
        let path = self.root.path().join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent");
        }
        fs::write(
            path,
            serde_json::to_vec_pretty(value).expect("serialize fixture"),
        )
        .expect("write fixture");
    }

    fn write_standard_contracts(&self) {
        self.write_json(
            "data/ci/validation-catalog.v1.json",
            &json!({
                "schema_version": 1,
                "policies": [
                    {"id": "pull_request", "included_tiers": ["fast", "pr"]},
                    {"id": "merge", "included_tiers": ["fast", "pr", "merge"]}
                ],
                "profiles": [
                    {"id": "kernel", "validators": ["rust-fmt", "rust-test"]},
                    {"id": "frontend", "validators": ["frontend-unit"]},
                    {"id": "release", "validators": ["release-package"]}
                ],
                "validators": [
                    {"id": "rust-fmt", "tier": "fast", "gate": "required", "platforms": ["ubuntu"], "trust": "untrusted_pr", "command_id": "cargo-fmt", "workflow_jobs": ["rust"]},
                    {"id": "rust-test", "tier": "pr", "gate": "required", "platforms": ["windows", "ubuntu"], "trust": "untrusted_pr", "command_id": "cargo-test", "workflow_jobs": ["rust"]},
                    {"id": "frontend-unit", "tier": "pr", "gate": "required", "platforms": ["ubuntu"], "trust": "untrusted_pr", "command_id": "npm-unit", "workflow_jobs": ["frontend"]},
                    {"id": "release-package", "tier": "release", "gate": "required", "platforms": ["windows"], "trust": "trusted", "command_id": "package", "workflow_jobs": ["release"]}
                ],
                "commands": [
                    {"id": "cargo-fmt", "program": "cargo", "args": ["fmt", "--check"]},
                    {"id": "cargo-test", "program": "cargo", "args": ["test"]},
                    {"id": "npm-unit", "program": "npm", "args": ["run", "test:unit"]},
                    {"id": "package", "program": "npm", "args": ["run", "tauri", "build"]}
                ]
            }),
        );
        self.write_json(
            "data/ci/impact-map.v1.json",
            &json!({
                "schema_version": 1,
                "supported_extensions": ["com.oclive.ci.fixture"],
                "module_bindings": [
                    {"module_id": "oclive.kernel", "descriptor": "data/ci/modules/kernel.oclive.module.json", "selectors": [{"kind": "prefix", "value": "kernel"}]},
                    {"module_id": "oclive.frontend", "descriptor": "data/ci/modules/frontend.oclive.module.json", "selectors": [{"kind": "prefix", "value": "distros/chat-pro"}, {"kind": "exact", "value": "vite.config.ts"}]}
                ],
                "policy_affects": {"oclive.kernel": ["oclive.frontend"]},
                "risk_overrides": [
                    {"id": "workspace-contract", "selectors": [{"kind": "exact", "value": "Cargo.toml"}], "full": true, "reason": "workspace membership changed"},
                    {"id": "frontend-config", "selectors": [{"kind": "exact", "value": "vite.config.ts"}], "force_profiles": ["frontend"], "reason": "shared frontend config changed"}
                ]
            }),
        );
        self.write_descriptor(
            "kernel",
            json!({
                "schema_version": 1,
                "module": {"id": "oclive.kernel", "kind": "kernel"},
                "provides": ["kernel.runtime"],
                "runtime_requires": [],
                "resource_claims": [],
                "declared_affects": [],
                "validation_profiles": ["kernel"],
                "platforms": ["windows", "linux"],
                "extensions": {}
            }),
        );
        self.write_descriptor(
            "frontend",
            json!({
                "schema_version": 1,
                "module": {"id": "oclive.frontend", "kind": "distro"},
                "provides": ["distro.chat"],
                "runtime_requires": ["kernel.runtime"],
                "resource_claims": [],
                "declared_affects": ["oclive.kernel"],
                "validation_profiles": ["frontend"],
                "platforms": ["windows", "linux"],
                "extensions": {}
            }),
        );
    }

    fn write_descriptor(&self, name: &str, value: Value) {
        self.write_json(
            &format!("data/ci/modules/{name}.oclive.module.json"),
            &value,
        );
    }

    fn planner(&self) -> Planner {
        Planner::load(
            self.root.path(),
            "data/ci/impact-map.v1.json",
            "data/ci/validation-catalog.v1.json",
        )
        .expect("load planner")
    }
}

fn request(changed_files: &[&str]) -> PlanRequest {
    PlanRequest {
        base_sha: "base".to_owned(),
        head_sha: "head".to_owned(),
        policy: "pull_request".to_owned(),
        shadow: true,
        changed_files: changed_files.iter().map(ToString::to_string).collect(),
    }
}

fn ids(values: &[ReasonedSelection]) -> Vec<&str> {
    values.iter().map(|value| value.id.as_str()).collect()
}

#[test]
fn path_binding_and_semantic_edges_form_cycle_safe_closure() {
    let fixture = Fixture::new();
    fixture.write_standard_contracts();
    let plan = fixture
        .planner()
        .plan(request(&["kernel/crates/thing/src/lib.rs"]))
        .expect("plan");

    assert!(!plan.fallback.full);
    assert_eq!(ids(&plan.direct_modules), vec!["oclive.kernel"]);
    assert_eq!(
        ids(&plan.affected_modules),
        vec!["oclive.frontend", "oclive.kernel"]
    );
    assert_eq!(ids(&plan.selected_profiles), vec!["frontend", "kernel"]);
    assert_eq!(
        plan.selected_validators
            .iter()
            .map(|value| value.id.as_str())
            .collect::<Vec<_>>(),
        vec!["frontend-unit", "rust-fmt", "rust-test"]
    );
}

#[test]
fn unknown_path_fails_safe_to_full_policy_without_release_work() {
    let fixture = Fixture::new();
    fixture.write_standard_contracts();
    let plan = fixture
        .planner()
        .plan(request(&["new-domain/file.txt"]))
        .expect("plan");

    assert!(plan.fallback.full);
    assert_eq!(
        ids(&plan.affected_modules),
        vec!["oclive.frontend", "oclive.kernel"]
    );
    assert!(plan
        .fallback
        .reasons
        .contains(&"unmapped_changed_path:new-domain/file.txt".to_owned()));
    assert_eq!(
        plan.selected_validators
            .iter()
            .map(|value| value.id.as_str())
            .collect::<Vec<_>>(),
        vec!["frontend-unit", "rust-fmt", "rust-test"]
    );
    assert_eq!(plan.skipped_validators[0].id, "release-package");
    assert_eq!(
        plan.skipped_validators[0].reason,
        "tier_not_in_policy:release"
    );
}

#[test]
fn unsupported_required_extension_forces_full_but_optional_only_warns() {
    let fixture = Fixture::new();
    fixture.write_standard_contracts();
    let descriptor_path = fixture
        .root
        .path()
        .join("data/ci/modules/kernel.oclive.module.json");
    let mut descriptor: Value =
        serde_json::from_slice(&fs::read(&descriptor_path).expect("read")).expect("parse");
    descriptor["extensions"] = json!({
        "com.example.optional": {"required": false, "config": {"x": 1}},
        "com.example.required": {"required": true, "config": {}}
    });
    fixture.write_descriptor("kernel", descriptor);

    let plan = fixture
        .planner()
        .plan(request(&["kernel/src/lib.rs"]))
        .expect("plan");
    assert!(plan.fallback.full);
    assert!(plan
        .fallback
        .reasons
        .iter()
        .any(|reason| reason.contains("unsupported_required_extension:com.example.required")));
    assert_eq!(plan.warnings.len(), 1);
    assert!(plan.warnings[0].contains("com.example.optional"));
}

#[test]
fn force_profile_risk_does_not_require_full_fallback() {
    let fixture = Fixture::new();
    fixture.write_standard_contracts();
    let plan = fixture
        .planner()
        .plan(request(&["vite.config.ts"]))
        .expect("plan");

    assert!(!plan.fallback.full);
    assert!(ids(&plan.selected_profiles).contains(&"frontend"));
}

#[test]
fn exact_workspace_risk_forces_full_fallback() {
    let fixture = Fixture::new();
    fixture.write_standard_contracts();
    let plan = fixture
        .planner()
        .plan(request(&["Cargo.toml"]))
        .expect("plan");

    assert!(plan.fallback.full);
    assert!(plan
        .fallback
        .reasons
        .iter()
        .any(|reason| reason.starts_with("risk_override:workspace-contract:")));
}

#[test]
fn plan_is_byte_stable_for_reordered_duplicate_input() {
    let fixture = Fixture::new();
    fixture.write_standard_contracts();
    let planner = fixture.planner();
    let first = planner
        .plan(request(&["kernel/z.rs", "kernel/a.rs", "kernel/z.rs"]))
        .expect("first");
    let second = planner
        .plan(request(&["kernel/a.rs", "kernel/z.rs"]))
        .expect("second");

    assert_eq!(
        serde_json::to_vec(&first).expect("serialize first"),
        serde_json::to_vec(&second).expect("serialize second")
    );
}

#[test]
fn descriptor_path_cannot_escape_repository() {
    let fixture = Fixture::new();
    fixture.write_standard_contracts();
    let map_path = fixture.root.path().join("data/ci/impact-map.v1.json");
    let mut map: Value =
        serde_json::from_slice(&fs::read(&map_path).expect("read map")).expect("parse map");
    map["module_bindings"][0]["descriptor"] = json!("../escape.json");
    fixture.write_json("data/ci/impact-map.v1.json", &map);

    let error = Planner::load(
        fixture.root.path(),
        "data/ci/impact-map.v1.json",
        "data/ci/validation-catalog.v1.json",
    )
    .expect_err("path traversal must fail");
    assert!(error.to_string().contains("repository-relative"));
}

#[test]
fn malformed_module_descriptor_is_a_full_fallback_issue() {
    let fixture = Fixture::new();
    fixture.write_standard_contracts();
    fs::write(
        fixture
            .root
            .path()
            .join("data/ci/modules/kernel.oclive.module.json"),
        b"{not-json",
    )
    .expect("write malformed descriptor");
    let plan = fixture
        .planner()
        .plan(request(&["kernel/src/lib.rs"]))
        .expect("plan");

    assert!(plan.fallback.full);
    assert!(plan
        .fallback
        .reasons
        .iter()
        .any(|reason| reason.starts_with("module_metadata:oclive.kernel:")));
}

#[test]
fn missing_module_descriptor_is_a_full_fallback_issue() {
    let fixture = Fixture::new();
    fixture.write_standard_contracts();
    fs::remove_file(
        fixture
            .root
            .path()
            .join("data/ci/modules/kernel.oclive.module.json"),
    )
    .expect("remove descriptor");
    let plan = fixture
        .planner()
        .plan(request(&["kernel/src/lib.rs"]))
        .expect("plan");

    assert!(plan.fallback.full);
    assert!(plan
        .fallback
        .reasons
        .iter()
        .any(|reason| reason.contains("descriptor_read_error:NotFound")));
}

#[test]
fn fixture_paths_are_repository_relative() {
    let fixture = Fixture::new();
    fixture.write_standard_contracts();
    assert!(Path::new("data/ci/impact-map.v1.json").is_relative());
    fixture.planner();
}

#[test]
fn bundled_json_schemas_are_valid_json_objects() {
    for schema in [
        include_str!("../schemas/oclive.module.schema.json"),
        include_str!("../schemas/impact-map.schema.json"),
        include_str!("../schemas/validation-catalog.schema.json"),
        include_str!("../schemas/ci-plan.schema.json"),
    ] {
        let value: Value = serde_json::from_str(schema).expect("schema must parse");
        assert_eq!(
            value["$schema"],
            "https://json-schema.org/draft/2020-12/schema"
        );
        assert_eq!(value["type"], "object");
    }
}

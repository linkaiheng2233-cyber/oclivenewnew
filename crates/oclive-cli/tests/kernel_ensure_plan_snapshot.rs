//! Golden contract: `oclive kernel ensure --plan-only --json` shape (shared with oclive-vscode).

use std::path::PathBuf;
use std::process::Command;

const REQUIRED_TOP_LEVEL: &[&str] = &[
    "schema_version",
    "plan",
    "profile_compat",
    "caller_requirements",
    "executed",
    "health_ok",
];

const REQUIRED_PLAN: &[&str] = &["action", "degraded"];

#[test]
fn kernel_ensure_plan_snapshot_fields() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root");
    let profile = repo_root.join("examples/distro-profiles/vscode.oclive.toml");
    assert!(profile.is_file(), "missing {}", profile.display());

    let bin = env!("CARGO_BIN_EXE_oclive-cli");
    let output = Command::new(bin)
        .args([
            "kernel",
            "ensure",
            "--plan-only",
            "--json",
            "--path",
        ])
        .arg(repo_root)
        .args([
            "--roles-dir",
            repo_root.join("roles").to_str().unwrap(),
            "--distro",
            "vscode",
            "--distro-profile",
        ])
        .arg(profile)
        .output()
        .expect("spawn oclive-cli");

    assert!(
        output.status.success(),
        "ensure failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let v: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("ensure stdout must be JSON");

    assert_eq!(v.get("schema_version").and_then(|x| x.as_u64()), Some(2));

    for key in REQUIRED_TOP_LEVEL {
        assert!(v.get(key).is_some(), "missing top-level field `{key}`");
    }
    let plan = v.get("plan").expect("plan");
    for key in REQUIRED_PLAN {
        assert!(plan.get(key).is_some(), "missing plan.{key}");
    }

    let golden_path = manifest_dir.join("tests/fixtures/kernel_ensure_plan_v1.json");
    let golden: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&golden_path).expect("read golden"))
            .expect("parse golden");
    assert_eq!(
        golden.get("schema_version"),
        v.get("schema_version"),
        "schema_version drift — update golden + oclive-vscode contract test"
    );
}

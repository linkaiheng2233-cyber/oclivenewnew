//! The foundation template must work outside the repository without a reference Host.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::fs;
use std::process::Command;

#[test]
fn generated_kernel_base_is_a_host_free_contract_project() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("foundation");
    let root = common::repo_root();
    let registry_home = temp.path().join("user-registry-must-not-exist");
    let init = common::cli_command()
        .env("OCLIVE_HOME", &registry_home)
        .args([
            "init",
            "--non-interactive",
            "--quiet",
            "--template",
            "kernel-base",
            "--project-name",
            "foundation-consumer",
            "--kernel-source",
            root.to_str().unwrap(),
            "-o",
            project.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(init.status.success(), "init: {init:?}");
    assert!(!registry_home.exists(), "foundation wrote user registry");
    assert_eq!(fs::read_dir(&project).unwrap().count(), 5);
    for folder in ["src", "tests"] {
        assert_eq!(fs::read_dir(project.join(folder)).unwrap().count(), 1);
    }
    for unwanted in [
        "roles",
        "plugins",
        "docs",
        "vendor",
        "monolith.toml",
        "CONFIG_REFERENCE.md",
        "src/main.rs",
    ] {
        assert!(!project.join(unwanted).exists(), "unexpected {unwanted}");
    }
    fs::copy(root.join("Cargo.lock"), project.join("Cargo.lock")).unwrap();
    let manifest = project.join("Cargo.toml");
    let test = Command::new("cargo")
        .current_dir(&root)
        .args(["test", "--offline", "--color", "never", "--manifest-path"])
        .arg(&manifest)
        .output()
        .unwrap();
    assert!(test.status.success(), "child Cargo: {test:?}");
    let stdout = String::from_utf8_lossy(&test.stdout);
    print!("{stdout}");
    assert!(stdout.contains("external_memory_uses_only_this_calls_material ... ok"));
    assert!(stdout.contains("unsupported_requirement_preserves_the_typed_failure ... ok"));
    let metadata = Command::new("cargo")
        .current_dir(&root)
        .args([
            "metadata",
            "--offline",
            "--format-version",
            "1",
            "--manifest-path",
        ])
        .arg(&manifest)
        .output()
        .unwrap();
    assert!(metadata.status.success(), "metadata: {metadata:?}");
    let graph: serde_json::Value = serde_json::from_slice(&metadata.stdout).unwrap();
    let packages = graph["packages"].as_array().unwrap();
    for forbidden in [
        "oclive_kernel_host",
        "oclive_kernel_runtime",
        "oclivenewnew-tauri",
        "tauri",
        "sqlx",
        "axum",
    ] {
        assert!(
            !packages.iter().any(|p| p["name"] == forbidden),
            "unexpected dependency: {forbidden}"
        );
    }
    let generated = packages
        .iter()
        .find(|p| p["name"] == "foundation-consumer")
        .unwrap();
    let dependencies = generated["dependencies"].as_array().unwrap();
    assert_eq!(dependencies.len(), 2);
    for required in ["oclive_kernel_contracts", "oclive_kernel_types"] {
        assert!(dependencies.iter().any(|d| d["name"] == required));
    }
    temp.close().unwrap();
}

#[test]
fn kernel_base_rejects_host_options_before_writing() {
    let temp = tempfile::tempdir().unwrap();
    let root = common::repo_root();
    for extra in [
        vec!["--monolith"],
        vec!["--backend-llm", "ollama"],
        vec!["--with-role-pack", "default"],
        vec!["--with-example-plugin"],
        vec!["--quick"],
        vec!["--smart"],
        vec!["--pipeline", "emotion-first"],
        vec!["--project-type", "kernel-server"],
        vec!["--preset", "minimal"],
        vec!["--dual-core"],
        vec!["--template-url", "https://invalid.example/template"],
    ] {
        let project = temp.path().join("must-not-exist");
        let mut args = vec![
            "init",
            "--non-interactive",
            "--template",
            "kernel-base",
            "--kernel-source",
            root.to_str().unwrap(),
            "-o",
            project.to_str().unwrap(),
        ];
        args.extend(extra);
        let result = common::run_cli_output(&args);
        assert!(!result.status.success(), "accepted {args:?}");
        assert!(String::from_utf8_lossy(&result.stderr).contains("kernel-base"));
        assert!(!project.exists(), "wrote output on rejection: {args:?}");
    }
    temp.close().unwrap();
}

#[test]
fn kernel_base_preview_check_and_failure_do_not_materialize_a_project() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("preview");
    let root = common::repo_root();
    let args = [
        "init",
        "--non-interactive",
        "--template",
        "kernel-base",
        "--kernel-source",
        root.to_str().unwrap(),
        "-o",
        project.to_str().unwrap(),
    ];
    let mut preview_args = args.to_vec();
    preview_args.extend(["--dry-run", "--json"]);
    let preview = common::run_cli_output(&preview_args);
    assert!(preview.status.success(), "preview: {preview:?}");
    let plan: serde_json::Value = serde_json::from_slice(&preview.stdout).unwrap();
    assert_eq!(plan["template"], "kernel-base");
    assert_eq!(plan["preset"], "none");
    assert_eq!(plan["monolith_enabled"], false);
    assert_eq!(
        plan["tree"],
        serde_json::json!([
            "preview/",
            "  Cargo.toml",
            "  src/lib.rs",
            "  tests/base_contracts.rs",
            "  README.md",
            "  .gitignore"
        ])
    );
    let mut check_args = args.to_vec();
    check_args.extend(["--check", "--json"]);
    let check = common::run_cli_output(&check_args);
    assert!(check.status.success(), "check: {check:?}");
    let items: serde_json::Value = serde_json::from_slice(&check.stdout).unwrap();
    assert!(items
        .as_array()
        .unwrap()
        .iter()
        .all(|item| item["status"] == "pass"));
    assert!(!project.exists());

    let missing_source = common::run_cli_output(&[
        "init",
        "--non-interactive",
        "--template",
        "kernel-base",
        "-o",
        project.to_str().unwrap(),
    ]);
    assert!(!missing_source.status.success());
    assert!(String::from_utf8_lossy(&missing_source.stderr).contains("requires --kernel-source"));
    assert!(!project.exists());
    fs::create_dir(&project).unwrap();
    fs::write(project.join("keep.txt"), b"owned-by-developer").unwrap();
    let occupied = common::run_cli_output(&args);
    assert!(!occupied.status.success());
    assert_eq!(
        fs::read(project.join("keep.txt")).unwrap(),
        b"owned-by-developer"
    );
    assert_eq!(fs::read_dir(&project).unwrap().count(), 1);
    temp.close().unwrap();
}

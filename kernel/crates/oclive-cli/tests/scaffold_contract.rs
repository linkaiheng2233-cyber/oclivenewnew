//! Stage 2A CLI contract: local discovery is diagnostic-only and fail-closed.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::{fs, path::Path, process::Command};

use serde_json::{json, Value};

fn write_manifest(project: &Path, permissions: Value) {
    let package = project.join(".oclive/scaffolds/example");
    fs::create_dir_all(&package).expect("create scaffold package");
    let manifest = json!({
        "schema_version": 1,
        "package": {
            "id": "dev.example.scaffold",
            "version": "1.2.3",
            "display_name": "Example scaffold",
            "description": "Local declaration-only fixture",
            "maintainer": "fixture maintainer"
        },
        "compatibility": {
            "oclive_cli": ">=0.1.0, <1.0.0",
            "scaffold_contract": "^1.0"
        },
        "command_namespace": "dev.example.tools",
        "generators": [{
            "id": "example",
            "kind": "project",
            "driver": { "kind": "instruction", "path": "instructions/create.md" }
        }],
        "commands": [],
        "permissions": permissions,
        "defaults": {},
        "dependencies": [],
        "extends": [],
        "composition": {},
        "extensions": {}
    });
    fs::write(
        package.join("oclive.scaffold.json"),
        serde_json::to_vec_pretty(&manifest).expect("serialize manifest"),
    )
    .expect("write manifest");
}

fn run_scaffold(project: &Path, home: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_oclive-cli"))
        .env("OCLIVE_HOME", home)
        .arg("scaffold")
        .args(args)
        .arg("-o")
        .arg(project)
        .output()
        .expect("run scaffold command")
}

#[test]
fn project_package_is_reported_untrusted_and_lock_records_source() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path().join("project");
    let home = temp.path().join("home");
    fs::create_dir_all(&project).expect("create project");
    write_manifest(&project, json!([]));

    let list = run_scaffold(&project, &home, &["list"]);
    assert!(
        list.status.success(),
        "{}",
        String::from_utf8_lossy(&list.stderr)
    );
    let stdout = String::from_utf8_lossy(&list.stdout);
    assert!(stdout.contains("dev.example.scaffold@1.2.3"));
    assert!(stdout.contains("untrusted_local_scaffold"));
    assert!(stdout.contains("cannot control CI"));

    let resolve = run_scaffold(&project, &home, &["resolve", "--write-lock", "--json"]);
    assert!(
        resolve.status.success(),
        "{}",
        String::from_utf8_lossy(&resolve.stderr)
    );
    let lock_path = project.join(".oclive/scaffold.lock.json");
    let lock: Value =
        serde_json::from_slice(&fs::read(lock_path).expect("read lock")).expect("parse lock");
    let package = lock["packages"]
        .as_array()
        .expect("packages")
        .iter()
        .find(|package| package["id"] == "dev.example.scaffold")
        .expect("custom package lock entry");
    assert_eq!(package["source"], "project");
    assert_eq!(package["trust"], "untrusted_local");
    assert_eq!(package["command_namespace"], "dev.example.tools");
}

#[test]
fn forbidden_ci_capability_fails_catalog_resolution() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path().join("project");
    let home = temp.path().join("home");
    fs::create_dir_all(&project).expect("create project");
    write_manifest(&project, json!(["ci.runner"]));

    let output = run_scaffold(&project, &home, &["list"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("forbidden_ci_capability"), "{stderr}");
    assert!(
        stderr.contains("cannot resolve scaffold catalog"),
        "{stderr}"
    );
}

#[test]
fn official_fallback_is_available_with_empty_local_sources() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path().join("project");
    let home = temp.path().join("home");
    fs::create_dir_all(&project).expect("create project");

    let output = run_scaffold(&project, &home, &["list", "--json"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let document: Value = serde_json::from_slice(&output.stdout).expect("parse list JSON");
    assert_eq!(
        document["resolved"]["packages"]
            .as_array()
            .expect("packages")
            .len(),
        4
    );
    assert!(document["resolved"]["packages"]
        .as_array()
        .expect("packages")
        .iter()
        .all(|package| package["source"] == "official"));
}

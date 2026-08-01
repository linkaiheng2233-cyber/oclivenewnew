//! Stage 2A CLI contract: local discovery is diagnostic-only and fail-closed.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::{fs, path::Path, process::Command};

use serde_json::{json, Value};

use oclive_scaffold::scaffold_sha256_hex;

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
    let mut command = scaffold_command(project, home);
    command
        .args(args)
        .arg("-o")
        .arg(project)
        .output()
        .expect("run scaffold command")
}

fn scaffold_command(_project: &Path, home: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_oclive-cli"));
    command.env("OCLIVE_HOME", home).arg("scaffold");
    command
}

fn write_generation_manifest(project: &Path) {
    let package = project.join(".oclive/scaffolds/generator");
    fs::create_dir_all(package.join("templates")).expect("generator package");
    let template = b"Hello, {{project_name}}!\n";
    fs::write(package.join("templates/README.md.tmpl"), template).expect("template");
    let instruction = json!({
        "schema_version": 1,
        "variables": {
            "project_name": {
                "description": "Generated project name",
                "required": true
            }
        },
        "files": [{
            "source": "templates/README.md.tmpl",
            "target": "README.md",
            "mode": "text",
            "sha256": scaffold_sha256_hex(template)
        }]
    });
    let instruction_bytes = serde_json::to_vec_pretty(&instruction).expect("instruction JSON");
    fs::write(package.join("instructions.json"), &instruction_bytes).expect("instruction");
    let manifest = json!({
        "schema_version": 1,
        "package": {
            "id": "dev.example.generator",
            "version": "1.0.0",
            "display_name": "Generator fixture",
            "description": "CLI generation fixture",
            "maintainer": "fixture maintainer"
        },
        "compatibility": {
            "oclive_cli": ">=0.1.0, <1.0.0",
            "scaffold_contract": ">=1.1, <2"
        },
        "command_namespace": "dev.example.generator",
        "generators": [{
            "id": "project",
            "kind": "project",
            "driver": {
                "kind": "instruction",
                "path": "instructions.json",
                "sha256": scaffold_sha256_hex(&instruction_bytes)
            }
        }],
        "commands": [],
        "permissions": ["project.write"],
        "defaults": {},
        "dependencies": [],
        "extends": [],
        "composition": {},
        "extensions": {}
    });
    fs::write(
        package.join("oclive.scaffold.json"),
        serde_json::to_vec_pretty(&manifest).expect("manifest JSON"),
    )
    .expect("manifest");
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

#[test]
fn locked_generation_requires_acknowledgement_and_writes_provenance() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path().join("project");
    let home = temp.path().join("home");
    let output = temp.path().join("generated");
    fs::create_dir_all(&project).expect("create project");
    write_generation_manifest(&project);

    let resolve = run_scaffold(&project, &home, &["resolve", "--write-lock"]);
    assert!(
        resolve.status.success(),
        "{}",
        String::from_utf8_lossy(&resolve.stderr)
    );

    let refused = scaffold_command(&project, &home)
        .args(["generate", "dev.example.generator", "project", "--output"])
        .arg(&output)
        .args(["--set", "project_name=Stage2B"])
        .arg("-o")
        .arg(&project)
        .output()
        .expect("run refused generation");
    assert!(!refused.status.success());
    assert!(String::from_utf8_lossy(&refused.stderr).contains("untrusted_confirmation_required"));
    assert!(!output.exists());

    let generated = scaffold_command(&project, &home)
        .args(["generate", "dev.example.generator", "project", "--output"])
        .arg(&output)
        .args([
            "--set",
            "project_name=Stage2B",
            "--accept-untrusted",
            "--json",
        ])
        .arg("-o")
        .arg(&project)
        .output()
        .expect("run generation");
    assert!(
        generated.status.success(),
        "{}",
        String::from_utf8_lossy(&generated.stderr)
    );
    let result: Value = serde_json::from_slice(&generated.stdout).expect("generation JSON");
    assert_eq!(result["provenance"]["generator_id"], "project");
    assert_eq!(
        fs::read_to_string(output.join("README.md")).expect("generated README"),
        "Hello, Stage2B!\n"
    );
    assert!(output.join(".oclive/scaffold.provenance.json").is_file());
}

#[test]
fn official_builtin_generation_returns_domain_delegation() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path().join("project");
    let home = temp.path().join("home");
    let output = temp.path().join("generated");
    fs::create_dir_all(&project).expect("create project");

    let result = scaffold_command(&project, &home)
        .args([
            "generate",
            "com.oclive.scaffold.plugin",
            "directory",
            "--output",
        ])
        .arg(&output)
        .arg("-o")
        .arg(&project)
        .output()
        .expect("run official delegation");
    assert!(!result.status.success());
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        stderr.contains("use `oclive plugin create --type directory`"),
        "{stderr}"
    );
    assert!(!output.exists());
}

//! Exercise the documented source SDK recipe in an author-owned workspace.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};

const SELECTION_TEST: &str =
    "tests::public_consumer_calls_the_external_selection_without_rewriting_material";

fn cargo(project: &Path, args: &[&str]) -> Output {
    Command::new("cargo")
        .current_dir(project)
        .args(args)
        .output()
        .expect("run Cargo in the temporary author project")
}

fn successful_cargo(project: &Path, args: &[&str]) -> Output {
    let output = cargo(project, args);
    assert!(
        output.status.success(),
        "Cargo {args:?} failed: {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

#[test]
fn source_sdk_memory_recipe_runs_outside_the_workspace() {
    let temp = tempfile::tempdir().unwrap();
    let root = common::repo_root();
    let project = temp.path().join("author memory");
    let isolated_home = temp.path().join("unused-cli-home");
    let generated = common::cli_command()
        .env("OCLIVE_HOME", &isolated_home)
        .args([
            "--experimental",
            "init",
            "--non-interactive",
            "--no-smart",
            "--template",
            "kernel-base",
            "--base-example",
            "none",
            "--license",
            "Apache-2.0",
            "--project-name",
            "sdk-memory-caller",
            "--kernel-source",
            root.to_str().unwrap(),
            "-o",
            project.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(generated.status.success(), "generate: {generated:?}");

    let example = root.join("kernel/crates/oclive_kernel_runtime/examples/external_memory_base");
    for (source, destination) in [
        (example.join("main.rs"), project.join("src/main.rs")),
        (example.join("memory.rs"), project.join("src/memory.rs")),
        (root.join("LICENSE"), project.join("LICENSE")),
        (
            root.join("examples/developer-sdk-memory/project-README.md"),
            project.join("README.md"),
        ),
        (root.join("Cargo.lock"), project.join("Cargo.lock")),
    ] {
        assert!(
            !destination.exists() || destination == project.join("README.md"),
            "refusing to overwrite {destination:?}"
        );
        fs::copy(&source, &destination).unwrap();
        assert_eq!(fs::read(&source).unwrap(), fs::read(&destination).unwrap());
    }
    let runtime = root.join("kernel/crates/oclive_kernel_runtime");
    successful_cargo(
        &project,
        &["add", "--offline", "--path", runtime.to_str().unwrap()],
    );
    let derived_lock = fs::read(project.join("Cargo.lock")).unwrap();
    let metadata_output = successful_cargo(
        &project,
        &["metadata", "--locked", "--offline", "--format-version", "1"],
    );
    let metadata: serde_json::Value = serde_json::from_slice(&metadata_output.stdout).unwrap();
    assert_eq!(
        Path::new(metadata["workspace_root"].as_str().unwrap())
            .canonicalize()
            .unwrap(),
        project.canonicalize().unwrap()
    );
    let members = metadata["workspace_members"].as_array().unwrap();
    assert_eq!(members.len(), 1);
    let packages = metadata["packages"].as_array().unwrap();
    let author = packages.iter().find(|p| p["id"] == members[0]).unwrap();
    assert_eq!(author["name"], "sdk-memory-caller");
    let dependencies: HashSet<_> = author["dependencies"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        dependencies,
        HashSet::from([
            "oclive_kernel_contracts",
            "oclive_kernel_types",
            "oclive_kernel_runtime"
        ])
    );
    let seed: toml::Value =
        toml::from_str(&fs::read_to_string(root.join("Cargo.lock")).unwrap()).unwrap();
    let third_party: HashSet<_> = seed["package"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|p| {
            p.get("source").and_then(|s| s.as_str()).map(|source| {
                (
                    p["name"].as_str().unwrap(),
                    p["version"].as_str().unwrap(),
                    source,
                )
            })
        })
        .collect();
    for package in packages {
        let name = package["name"].as_str().unwrap();
        if dependencies.contains(name) {
            assert_eq!(
                Path::new(package["manifest_path"].as_str().unwrap())
                    .canonicalize()
                    .unwrap(),
                root.join("kernel/crates")
                    .join(name)
                    .join("Cargo.toml")
                    .canonicalize()
                    .unwrap(),
                "source dependency must use this checkout: {name}"
            );
        }
        assert!(
            name != "oclive_kernel_host"
                && !name.starts_with("tauri")
                && !["tao", "wry"].contains(&name),
            "unexpected Host/desktop dependency: {name}"
        );
        if let Some(source) = package["source"].as_str() {
            assert!(
                third_party.contains(&(name, package["version"].as_str().unwrap(), source)),
                "dependency identity absent from the source lock: {name}"
            );
        }
    }
    successful_cargo(&project, &["check", "--locked", "--offline"]);
    let tests = successful_cargo(
        &project,
        &["test", "--locked", "--offline", "--color", "never"],
    );
    let test_text = String::from_utf8(tests.stdout).unwrap();
    assert!(test_text.contains(&format!("{SELECTION_TEST} ... ok")));
    assert!(test_text.contains("3 passed; 0 failed"), "{test_text}");
    let run = successful_cargo(&project, &["run", "--locked", "--offline"]);
    let run_text = String::from_utf8(run.stdout).unwrap();
    assert_eq!(
        run_text.lines().next(),
        Some("external memory: selected=2 baseline=3 memory_calls=1 llm_calls=0")
    );
    assert_eq!(derived_lock, fs::read(project.join("Cargo.lock")).unwrap());
    assert!(!isolated_home.exists(), "recipe wrote CLI user state");

    // Mutate only the author copy. A real behavior failure must be distinguishable
    // from a build/toolchain failure; merely seeing a nonzero exit is insufficient.
    let memory_path = project.join("src/memory.rs");
    let memory = fs::read_to_string(&memory_path).unwrap();
    assert_eq!(memory.matches(".take(self.max_results)").count(), 1);
    fs::write(
        &memory_path,
        memory.replace(".take(self.max_results)", ".take(0)"),
    )
    .unwrap();
    let negative = cargo(
        &project,
        &[
            "test",
            "--locked",
            "--offline",
            "--color",
            "never",
            "--bin",
            "sdk-memory-caller",
            SELECTION_TEST,
            "--",
            "--exact",
        ],
    );
    assert!(!negative.status.success(), "mutated Memory passed");
    let negative_text = String::from_utf8(negative.stdout).unwrap();
    assert!(negative_text.contains(&format!("{SELECTION_TEST} ... FAILED")));
    assert!(negative_text.contains("0 passed; 1 failed"));
    assert_eq!(derived_lock, fs::read(project.join("Cargo.lock")).unwrap());
    temp.close().unwrap();
}

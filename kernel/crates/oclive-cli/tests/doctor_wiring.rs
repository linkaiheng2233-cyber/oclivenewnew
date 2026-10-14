//! Project declarations are not runtime binding or call evidence.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

const PACKAGE: &str = "[package]\nname = \"memory-view-case\"\nversion = \"0.1.0\"\n";
const DECLARATION: &str = r#"
[package.metadata.oclive.wiring]
schema_version = 1
call_scope = ["memory"]

[package.metadata.oclive.wiring.slots.memory]
implementation = "RecentLiteralMemory"
via = "MinimalRoleBaseConsumer"
scope = "project"
"#;

fn view(root: &Path, extra: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_oclive-cli"))
        .current_dir(root)
        .args(["doctor", "wiring"])
        .args(extra)
        // A declaration view must not need cargo, rustc, probes or a Host.
        .env("PATH", root)
        .output()
        .unwrap()
}

fn text(output: &Output) -> String {
    assert!(output.status.success(), "{output:?}");
    String::from_utf8(output.stdout.clone()).unwrap()
}

#[test]
fn rereads_memory_declaration_and_separates_interfaces_from_implementations() {
    let temp = tempfile::tempdir().unwrap();
    let manifest = temp.path().join("Cargo.toml");
    let first = format!("{PACKAGE}{DECLARATION}");
    fs::write(&manifest, &first).unwrap();
    let output = text(&view(temp.path(), &[]));
    assert!(output.contains("Project: memory-view-case"));
    assert!(output.contains("package.metadata.oclive.wiring"));
    assert!(output.contains("MemoryBase -- declared --> RecentLiteralMemory"));
    assert!(output.contains("oclive_kernel_contracts::MemoryBase"));
    assert!(output.contains("Entry declaration: MinimalRoleBaseConsumer"));
    assert!(output.contains("Host verification: not connected"));
    assert!(output.contains("Binding / call / target functionality: not verified"));
    assert!(output.contains("EmotionBase -- implementation not provided"));
    assert!(output.contains("Call scope declaration: memory (not a call log)"));
    assert!(output.contains("not execution order"));
    for base in [
        "MemoryBase",
        "EmotionBase",
        "EventBase",
        "PromptBase",
        "LlmBase",
        "AgentBase",
    ] {
        assert!(output.contains(base), "missing interface {base}");
    }
    assert_eq!(fs::read_to_string(&manifest).unwrap(), first);
    let second = first.replace("RecentLiteralMemory", "AuthorMemoryV2");
    fs::write(&manifest, &second).unwrap();
    let output = text(&view(temp.path(), &[]));
    assert!(output.contains("MemoryBase -- declared --> AuthorMemoryV2"));
    assert!(!output.contains("RecentLiteralMemory"));
    assert_eq!(fs::read_to_string(&manifest).unwrap(), second);
    let teaching = second
        .replace("scope = \"project\"", "scope = \"example-test\"")
        .replace("call_scope = [\"memory\"]", "call_scope = []");
    fs::write(&manifest, &teaching).unwrap();
    let output = text(&view(temp.path(), &[]));
    assert!(output.contains("example-test (not a runtime binding)"));
    assert!(output.contains("Call scope declaration: none declared (not a call log)"));
    assert!(output.contains("Binding / call / target functionality: not verified"));
    assert_eq!(fs::read_to_string(&manifest).unwrap(), teaching);
    assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 1);
}

#[test]
fn absent_metadata_stays_unknown_even_when_rust_contains_an_implementation() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("Cargo.toml"), PACKAGE).unwrap();
    fs::create_dir(temp.path().join("src")).unwrap();
    fs::write(
        temp.path().join("src/lib.rs"),
        "impl MemoryBase for SecretMemory {}\n",
    )
    .unwrap();
    let output = text(&view(temp.path(), &[]));
    assert!(output.contains("Wiring metadata: not provided"));
    assert!(output.contains("MemoryBase -- implementation not provided"));
    assert!(output.contains("Call scope declaration: not provided"));
    assert!(!output.contains("SecretMemory"));
    assert!(!temp.path().join("target").exists());
}

#[test]
fn unrelated_metadata_and_parse_failures_do_not_print_private_contents() {
    let temp = tempfile::tempdir().unwrap();
    let manifest = temp.path().join("Cargo.toml");
    let private = "PRIVATE_API_TOKEN_92761";
    let contents =
        format!("{PACKAGE}\n[package.metadata.private]\napi_token = \"{private}\"\n{DECLARATION}");
    fs::write(&manifest, &contents).unwrap();
    let output = view(temp.path(), &[]);
    assert!(text(&output).contains("RecentLiteralMemory"));
    assert!(!String::from_utf8_lossy(&output.stdout).contains(private));
    assert!(!String::from_utf8_lossy(&output.stderr).contains(private));
    fs::write(&manifest, format!("{PACKAGE}\napi_token = \"{private}")).unwrap();
    let output = view(temp.path(), &[]);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(!String::from_utf8_lossy(&output.stderr).contains(private));
    assert!(String::from_utf8_lossy(&output.stderr).contains("invalid Cargo.toml"));
}

#[test]
fn invalid_declarations_cannot_claim_verified_status_or_inject_terminal_text() {
    let temp = tempfile::tempdir().unwrap();
    let manifest = temp.path().join("Cargo.toml");
    for declaration in [
        "[package.metadata.oclive.wiring]\nschema_version = 2\n",
        "[package.metadata.oclive.wiring]\nschema_version = \"1\"\n",
        "[package.metadata.oclive.wiring]\nschema_version = 1\nverified = true\n",
        "[package.metadata.oclive.wiring]\nschema_version = 1\ncall_scope = [\"voice\"]\n",
        "[package.metadata.oclive.wiring]\nschema_version = 1\n[package.metadata.oclive.wiring.slots.voice]\nimplementation = \"Voice\"\n",
        "[package.metadata.oclive.wiring]\nschema_version = 1\n[package.metadata.oclive.wiring.slots.memory]\nimplementation = 42\n",
        "[package.metadata.oclive.wiring]\nschema_version = 1\n[package.metadata.oclive.wiring.slots.memory]\nimplementation = \"\"\n",
        "[package.metadata.oclive.wiring]\nschema_version = 1\n[package.metadata.oclive.wiring.slots.memory]\nscope = \"bound\"\n",
        "[package.metadata.oclive.wiring]\nschema_version = 1\n[package.metadata.oclive.wiring.slots.memory]\nimplementation = \"PRIVATE\\nVERIFIED\"\n",
    ] {
        let contents = format!("{PACKAGE}{declaration}");
        fs::write(&manifest, &contents).unwrap();
        let output = view(temp.path(), &[]);
        assert!(!output.status.success(), "accepted {declaration:?}");
        assert!(output.stdout.is_empty());
        assert!(!String::from_utf8_lossy(&output.stderr).contains("PRIVATE"));
        assert_eq!(fs::read_to_string(&manifest).unwrap(), contents);
    }
}

#[test]
fn missing_unreadable_and_workspace_only_sources_fail_without_output() {
    let temp = tempfile::tempdir().unwrap();
    for contents in [None, Some("[workspace]\n"), Some("[package]\nname = 42\n")] {
        let path = temp.path().join("Cargo.toml");
        if let Some(contents) = contents {
            fs::write(&path, contents).unwrap();
        }
        let output = view(temp.path(), &[]);
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
    let output = view(
        temp.path(),
        &["--manifest-path", temp.path().to_str().unwrap()],
    );
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
}

#[test]
fn environment_actions_cannot_be_combined_with_the_read_only_view() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(
        temp.path().join("Cargo.toml"),
        format!("{PACKAGE}{DECLARATION}"),
    )
    .unwrap();
    for flag in ["--fix", "--watch", "--sbom", "--yes", "--json"] {
        let output = Command::new(env!("CARGO_BIN_EXE_oclive-cli"))
            .current_dir(temp.path())
            .args(["doctor", flag, "wiring"])
            .env("PATH", temp.path())
            .output()
            .unwrap();
        assert!(!output.status.success(), "ignored {flag}");
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("read-only terminal"));
    }
    assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 1);
}

#[test]
fn explicit_manifest_path_works_without_executing_project_code() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("project");
    fs::create_dir(&project).unwrap();
    let manifest = project.join("Cargo.toml");
    fs::write(&manifest, format!("{PACKAGE}{DECLARATION}")).unwrap();
    fs::write(
        project.join("build.rs"),
        "compile_error!(\"must not build\");",
    )
    .unwrap();
    let output = view(
        temp.path(),
        &["--manifest-path", manifest.to_str().unwrap()],
    );
    assert!(text(&output).contains("RecentLiteralMemory"));
    assert_eq!(fs::read_dir(&project).unwrap().count(), 2);
    assert!(!project.join("target").exists());
}

#[test]
fn scaffold_initializes_empty_metadata_without_claiming_test_bindings() {
    let temp = tempfile::tempdir().unwrap();
    let source = common::repo_root();
    for example in ["memory", "none"] {
        let project = temp.path().join(example);
        let output = common::run_cli_output(&[
            "init",
            "--non-interactive",
            "--no-smart",
            "--template",
            "kernel-base",
            "--base-example",
            example,
            "--kernel-source",
            source.to_str().unwrap(),
            "-o",
            project.to_str().unwrap(),
        ]);
        assert!(output.status.success(), "init {example}: {output:?}");
        let contents = fs::read_to_string(project.join("Cargo.toml")).unwrap();
        let manifest: toml::Value = toml::from_str(&contents).unwrap();
        assert_eq!(
            manifest["package"]["metadata"]["oclive"]["wiring"]["schema_version"].as_integer(),
            Some(1)
        );
        assert_eq!(manifest["dependencies"].as_table().unwrap().len(), 2);
        let output = text(&view(&project, &[]));
        assert!(output.contains("Wiring metadata: provided"));
        assert!(output.contains("MemoryBase -- implementation not provided"));
        assert!(!output.contains("TestMemory"));
        assert_eq!(
            fs::read_to_string(project.join("Cargo.toml")).unwrap(),
            contents
        );
    }
}

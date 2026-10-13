//! The foundation template must work outside the repository without a reference Host.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::fs;
use std::process::Command;

/// 当前宿主目标三元组。
///
/// `cargo metadata` 默认解析**全平台**依赖图，于是会要求本机永远不会构建的
/// crate 也存在于本地缓存（例如 Linux 上的 `android_system_properties`，
/// 由 `chrono` → `iana-time-zone` 引入，只对 Android 生效）。配合 `--offline`
/// 时这类解析会直接失败。`--filter-platform` 把解析范围收敛到宿主平台，
/// 既保留"生成项目不需要联网"的断言，又不再索要跨平台 crate。
fn host_triple() -> String {
    let out = Command::new("rustc").args(["-vV"]).output().unwrap();
    assert!(out.status.success(), "rustc -vV: {out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    stdout
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .expect("rustc -vV should report a host triple")
        .trim()
        .to_string()
}

#[test]
fn generated_blank_foundation_keeps_all_six_contracts_available() {
    let temp = tempfile::tempdir().unwrap();
    let root = common::repo_root();
    let project = temp.path().join("blank");
    let args = [
        "init",
        "--non-interactive",
        "--template",
        "kernel-base",
        "--base-example",
        "none",
        "--kernel-source",
        root.to_str().unwrap(),
        "-o",
        project.to_str().unwrap(),
    ];
    let mut preview_args = args.to_vec();
    preview_args.extend(["--dry-run", "--json"]);
    let preview = common::run_cli_output(&preview_args);
    assert!(preview.status.success(), "blank preview: {preview:?}");
    let plan: serde_json::Value = serde_json::from_slice(&preview.stdout).unwrap();
    assert_eq!(
        plan["tree"],
        serde_json::json!([
            "blank/",
            "  Cargo.toml",
            "  src/lib.rs",
            "  README.md",
            "  .gitignore"
        ])
    );
    assert!(!project.exists());
    let init = common::run_cli_output(&args);
    assert!(init.status.success(), "blank init: {init:?}");
    assert_eq!(fs::read_dir(&project).unwrap().count(), 4);
    assert!(!project.join("tests").exists());
    let manifest: toml::Value =
        toml::from_str(&fs::read_to_string(project.join("Cargo.toml")).unwrap()).unwrap();
    assert_eq!(
        manifest["package"]["metadata"]["oclive"]["wiring"]["schema_version"].as_integer(),
        Some(1)
    );
    assert!(manifest["package"]["metadata"]["oclive"]["wiring"]
        .get("slots")
        .is_none());
    let readme = fs::read_to_string(project.join("README.md")).unwrap();
    assert!(readme.contains("空白契约工程"));
    assert!(!readme.contains("## 第一条开发练习"));
    fs::write(project.join("src/lib.rs"), r#"use oclive_kernel_contracts::{AgentBase, EmotionBase, EventBase, LlmBase, MemoryBase, PromptBase};

pub fn accepts_all_six(
    _memory: &dyn MemoryBase,
    _emotion: &dyn EmotionBase,
    _event: &dyn EventBase,
    _prompt: &dyn PromptBase,
    _llm: &dyn LlmBase,
    _agent: &dyn AgentBase,
) {
}
"#).unwrap();
    fs::copy(root.join("Cargo.lock"), project.join("Cargo.lock")).unwrap();
    let result = Command::new("cargo")
        .current_dir(&project)
        .args(["test", "--offline", "--color", "never"])
        .output()
        .unwrap();
    assert!(result.status.success(), "six public traits: {result:?}");
    temp.close().unwrap();
}

#[test]
fn foundation_example_options_are_not_silently_used_by_runtime_recipes() {
    let temp = tempfile::tempdir().unwrap();
    let root = common::repo_root();
    let project = temp.path().join("rejected");
    for extra in [
        vec!["--template", "kernel-base", "--base-example", "emotion"],
        vec!["--template", "headless-api", "--base-example", "none"],
        vec!["--base-example", "none", "--quick"],
        vec![
            "--template",
            "kernel-base",
            "--base-example",
            "none",
            "--backend-memory",
            "none",
        ],
    ] {
        let mut args = vec![
            "init",
            "--non-interactive",
            "--kernel-source",
            root.to_str().unwrap(),
            "-o",
            project.to_str().unwrap(),
        ];
        args.extend(extra);
        let result = common::run_cli_output(&args);
        assert!(!result.status.success(), "accepted {args:?}");
        assert!(!project.exists(), "wrote on invalid option {args:?}");
    }
    temp.close().unwrap();
}

#[test]
fn developer_owned_base_template_generates_and_detects_unreviewed_edits() {
    let temp = tempfile::tempdir().unwrap();
    let root = common::repo_root();
    let source = root.join("examples/scaffold-kernel-base-custom");
    let project = temp.path().join("author-project");
    let scaffolds = project.join(".oclive/scaffolds");
    for relative in [
        "refresh-digests.mjs",
        "package/oclive.scaffold.json",
        "package/instructions.json",
        "package/templates/Cargo.toml.tmpl",
        "package/templates/lib.rs.tmpl",
        "package/templates/README.md.tmpl",
        "package/templates/gitignore.tmpl",
    ] {
        let destination = scaffolds.join(relative);
        fs::create_dir_all(destination.parent().unwrap()).unwrap();
        fs::copy(source.join(relative), destination).unwrap();
    }
    let home = temp.path().join("isolated-home");
    let scaffold = |args: &[&str]| {
        common::cli_command()
            .env("OCLIVE_HOME", &home)
            .arg("scaffold")
            .args(args)
            .args(["--path", project.to_str().unwrap()])
            .output()
            .unwrap()
    };
    let resolved = scaffold(&["resolve", "--write-lock"]);
    assert!(resolved.status.success(), "lock: {resolved:?}");
    let output = temp.path().join("custom-output");
    let kernel_source = format!("kernel_source={}", root.display()).replace('\\', "/");
    let generate = [
        "generate",
        "dev.example.kernel-base",
        "project",
        "--output",
        output.to_str().unwrap(),
        "--set",
        "project_name=custom-base",
        "--set",
        &kernel_source,
        "--set",
        "module_goal=Memory experiment",
        "--accept-untrusted",
    ];
    let mut preview_args = generate.to_vec();
    preview_args.push("--dry-run");
    let preview = scaffold(&preview_args);
    assert!(preview.status.success(), "preview: {preview:?}");
    assert!(!output.exists());
    let result = scaffold(&generate);
    assert!(result.status.success(), "generate: {result:?}");
    assert!(fs::read_to_string(output.join("README.md"))
        .unwrap()
        .contains("Memory experiment"));
    assert!(output.join(".oclive/scaffold.provenance.json").exists());
    fs::copy(root.join("Cargo.lock"), output.join("Cargo.lock")).unwrap();
    let check = Command::new("cargo")
        .current_dir(&output)
        .args(["check", "--offline", "--color", "never"])
        .output()
        .unwrap();
    assert!(check.status.success(), "custom Cargo: {check:?}");

    let edited = scaffolds.join("package/templates/README.md.tmpl");
    let mut text = fs::read_to_string(&edited).unwrap();
    text.push_str("\nAuthor's new template note.\n");
    fs::write(&edited, text).unwrap();
    let second = temp.path().join("custom-second");
    let second_args = [
        "generate",
        "dev.example.kernel-base",
        "project",
        "--output",
        second.to_str().unwrap(),
        "--set",
        "project_name=custom-second",
        "--set",
        &kernel_source,
        "--set",
        "module_goal=Event experiment",
        "--accept-untrusted",
    ];
    let stale = scaffold(&second_args);
    assert!(
        !stale.status.success(),
        "unchecked edit accepted: {stale:?}"
    );
    assert!(!second.exists());
    // Fixture preparation uses the contract's digest utility; Rust E2E needs no Node runtime.
    let instruction_path = scaffolds.join("package/instructions.json");
    let mut instruction: serde_json::Value =
        serde_json::from_slice(&fs::read(&instruction_path).unwrap()).unwrap();
    for file in instruction["files"].as_array_mut().unwrap() {
        let bytes = fs::read(
            scaffolds
                .join("package")
                .join(file["source"].as_str().unwrap()),
        )
        .unwrap();
        file["sha256"] = oclive_scaffold::scaffold_sha256_hex(&bytes).into();
    }
    let instruction_bytes = serde_json::to_vec_pretty(&instruction).unwrap();
    fs::write(instruction_path, &instruction_bytes).unwrap();
    let manifest_path = scaffolds.join("package/oclive.scaffold.json");
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
    manifest["generators"][0]["driver"]["sha256"] =
        oclive_scaffold::scaffold_sha256_hex(&instruction_bytes).into();
    fs::write(manifest_path, serde_json::to_vec_pretty(&manifest).unwrap()).unwrap();
    let old_lock = scaffold(&second_args);
    assert!(
        !old_lock.status.success(),
        "stale lock accepted: {old_lock:?}"
    );
    assert!(!second.exists());
    assert!(scaffold(&["resolve", "--write-lock"]).status.success());
    let regenerated = scaffold(&second_args);
    assert!(regenerated.status.success(), "regenerate: {regenerated:?}");
    let readme = fs::read_to_string(second.join("README.md")).unwrap();
    assert!(readme.contains("Event experiment"));
    assert!(readme.contains("Author's new template note."));
    assert!(!home.exists(), "custom template wrote user state");
    temp.close().unwrap();
}

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
            "--filter-platform",
        ])
        .arg(host_triple())
        .arg("--manifest-path")
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

#[test]
fn the_readme_memory_exercise_observes_an_implementation_change() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("author-change");
    let root = common::repo_root();
    let init = common::run_cli_output(&[
        "init",
        "--non-interactive",
        "--quiet",
        "--template",
        "kernel-base",
        "--kernel-source",
        root.to_str().unwrap(),
        "-o",
        project.to_str().unwrap(),
    ]);
    assert!(init.status.success(), "init: {init:?}");
    fs::copy(root.join("Cargo.lock"), project.join("Cargo.lock")).unwrap();
    let run_tests = || {
        Command::new("cargo")
            .current_dir(&root)
            .args(["test", "--offline", "--color", "never", "--manifest-path"])
            .arg(project.join("Cargo.toml"))
            .output()
            .unwrap()
    };
    let original_result = run_tests();
    assert!(
        original_result.status.success(),
        "original: {original_result:?}"
    );
    let test_path = project.join("tests/base_contracts.rs");
    let original = fs::read_to_string(&test_path)
        .unwrap()
        .replace("\r\n", "\n");
    let suffix = original.replace(
        ".filter(|item| item.contains(needle))",
        ".filter(|item| item.ends_with(needle))",
    );
    assert_ne!(
        suffix, original,
        "README exercise could not find the implementation"
    );
    fs::write(&test_path, &suffix).unwrap();
    let changed_result = run_tests();
    let changed_stdout = String::from_utf8_lossy(&changed_result.stdout);
    println!("README intentional behavior failure:\n{changed_stdout}");
    assert!(
        !changed_result.status.success(),
        "changed implementation was not observed"
    );
    assert!(changed_stdout.contains("1 passed; 1 failed"));
    assert!(changed_stdout.contains("external_memory_uses_only_this_calls_material ... FAILED"));
    assert!(changed_stdout.contains("unsupported_requirement_preserves_the_typed_failure ... ok"));
    assert!(changed_stdout.contains("left: []"));
    let old_expectation = "\n        second\n";
    assert_eq!(suffix.matches(old_expectation).count(), 1);
    let accepted = suffix.replace(old_expectation, "\n        Vec::<String>::new()\n");
    fs::write(&test_path, accepted).unwrap();
    let accepted_result = run_tests();
    assert!(
        accepted_result.status.success(),
        "new agreement: {accepted_result:?}"
    );
    let accepted_stdout = String::from_utf8_lossy(&accepted_result.stdout);
    println!("README agreed behavior:\n{accepted_stdout}");
    assert!(accepted_stdout.contains("2 passed; 0 failed"));
    temp.close().unwrap();
}

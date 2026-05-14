//! 端到端：非交互生成项目并 `cargo build`。

use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/oclive-cli")
        .parent()
        .expect("repo root")
        .to_path_buf()
}

fn run_cli(args: &[&str]) -> std::process::ExitStatus {
    Command::new("cargo")
        .current_dir(repo_root())
        .args(["run", "-p", "oclive-cli", "--quiet", "--"])
        .args(args)
        .status()
        .expect("cargo run -p oclive-cli")
}

fn cargo_build(project_dir: &std::path::Path) -> std::process::ExitStatus {
    Command::new("cargo")
        .arg("build")
        .current_dir(project_dir)
        .status()
        .expect("spawn cargo build")
}

fn cargo_build_release(project_dir: &std::path::Path) -> std::process::ExitStatus {
    Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(project_dir)
        .status()
        .expect("spawn cargo build --release")
}

fn cargo_build_release_monolith(project_dir: &std::path::Path) -> std::process::ExitStatus {
    Command::new("cargo")
        .args(["build", "--release", "--features", "monolith"])
        .current_dir(project_dir)
        .status()
        .expect("spawn cargo build --release --features monolith")
}

fn release_binary_path(project_dir: &std::path::Path, bin_base: &str) -> std::path::PathBuf {
    let p = project_dir.join("target/release").join(bin_base);
    if cfg!(windows) {
        p.with_extension("exe")
    } else {
        p
    }
}

#[test]
fn e2e_preset_minimal_builds() {
    let tmp = tempfile::tempdir().unwrap();
    let out = tmp.path().join("k1");
    let st = run_cli(&[
        "init",
        "--non-interactive",
        "--quiet",
        "--preset",
        "minimal",
        "-o",
        out.to_str().unwrap(),
    ]);
    assert!(st.success(), "oclive-cli init");
    assert!(out.join("CONFIG_REFERENCE.md").is_file());
    let cref = fs::read_to_string(out.join("CONFIG_REFERENCE.md")).expect("CONFIG_REFERENCE.md");
    assert!(
        cref.contains("开发者编译选项"),
        "CONFIG_REFERENCE should mention planned developer compile options"
    );
    assert!(
        cref.contains("RFC_OCLIVE_MONOLITH_MODE"),
        "CONFIG_REFERENCE should link Monolith RFC"
    );
    assert!(
        !out.join("monolith.toml").exists(),
        "minimal preset without --monolith must not emit monolith.toml"
    );
    let settings_path = out.join("roles/default/settings.json");
    let raw = fs::read_to_string(&settings_path).expect("settings.json");
    let v: Value = serde_json::from_str(&raw).expect("parse settings");
    let pb = v.get("plugin_backends").unwrap().as_object().unwrap();
    assert!(
        !pb.contains_key("agent"),
        "minimal preset omits agent in JSON"
    );
    assert_eq!(pb.get("llm").unwrap().as_str().unwrap(), "ollama");
    for k in [
        "_comment_memory",
        "_comment_llm",
        "_comment_plugin_backends",
    ] {
        assert!(v.get(k).is_some(), "missing root key {k}");
    }
    for k in [
        "memory",
        "emotion",
        "event",
        "prompt",
        "llm",
        "complex_emotion",
    ] {
        assert!(pb.get(k).is_some(), "missing plugin_backends.{k}");
    }
    let st2 = cargo_build(&out);
    assert!(st2.success(), "generated project cargo build");
}

#[test]
fn e2e_preset_full_builds() {
    let tmp = tempfile::tempdir().unwrap();
    let out = tmp.path().join("k2");
    let st = run_cli(&[
        "init",
        "--non-interactive",
        "--quiet",
        "--preset",
        "full",
        "-o",
        out.to_str().unwrap(),
    ]);
    assert!(st.success());
    assert!(out.join("CONFIG_REFERENCE.md").is_file());
    let raw = fs::read_to_string(out.join("roles/default/settings.json")).unwrap();
    let v: Value = serde_json::from_str(&raw).unwrap();
    let pb = v.get("plugin_backends").unwrap().as_object().unwrap();
    assert_eq!(pb.get("llm").unwrap().as_str().unwrap(), "remote");
    assert_eq!(pb.get("agent").unwrap().as_str().unwrap(), "builtin");
    assert!(cargo_build(&out).success());
}

#[test]
fn e2e_preset_mixed_library_builds() {
    let tmp = tempfile::tempdir().unwrap();
    let out = tmp.path().join("k3");
    let st = run_cli(&[
        "init",
        "--non-interactive",
        "--quiet",
        "--preset",
        "mixed",
        "--project-type",
        "library",
        "-o",
        out.to_str().unwrap(),
    ]);
    assert!(st.success());
    assert!(cargo_build(&out).success());
}

#[test]
fn e2e_non_interactive_monolith_full_release_builds() {
    let tmp = tempfile::tempdir().unwrap();
    let out = tmp.path().join("mono_rel");
    let st = run_cli(&[
        "init",
        "--non-interactive",
        "--quiet",
        "--preset",
        "full",
        "--monolith",
        "-o",
        out.to_str().unwrap(),
    ]);
    assert!(st.success(), "oclive-cli init --monolith");
    let mt_path = out.join("monolith.toml");
    assert!(mt_path.is_file());
    let mt = fs::read_to_string(&mt_path).unwrap();
    assert!(mt.contains("enabled = true"));
    assert!(mt.contains("weld_modules = []"));
    assert!(mt.contains("exclude = []"));
    assert!(out.join("src/process_message_monolith.rs").is_file());
    assert!(
        cargo_build_release(&out).success(),
        "standard release build"
    );
    assert!(
        cargo_build_release_monolith(&out).success(),
        "monolith release build"
    );
    let slug = "my-oclive-kernel";
    let std_bin = release_binary_path(&out, slug);
    let mono_bin = release_binary_path(&out, &format!("{slug}-monolith"));
    assert!(std_bin.is_file(), "expected {:?}", std_bin);
    assert!(mono_bin.is_file(), "expected {:?}", mono_bin);
}

#[test]
fn e2e_monolith_ignored_for_library() {
    let tmp = tempfile::tempdir().unwrap();
    let out = tmp.path().join("mono_lib");
    assert!(run_cli(&[
        "init",
        "--non-interactive",
        "--quiet",
        "--preset",
        "full",
        "--monolith",
        "--project-type",
        "library",
        "-o",
        out.to_str().unwrap(),
    ])
    .success());
    assert!(
        !out.join("monolith.toml").exists(),
        "library project must not emit monolith.toml"
    );
}

#[test]
fn e2e_non_interactive_minimal_no_extra_input() {
    let tmp = tempfile::tempdir().unwrap();
    let out = tmp.path().join("test-project");
    assert!(run_cli(&[
        "init",
        "--non-interactive",
        "--quiet",
        "--preset",
        "minimal",
        "-o",
        out.to_str().unwrap(),
    ])
    .success());
    assert!(cargo_build(&out).success());
}

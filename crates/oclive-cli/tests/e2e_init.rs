//! 端到端：非交互生成项目并 `cargo build`。

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

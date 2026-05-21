#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn run_cli(args: &[&str]) -> std::process::Output {
    Command::new("cargo")
        .current_dir(repo_root())
        .args(["run", "-p", "oclive-cli", "--quiet", "--"])
        .args(args)
        .env("OCLIVE_ROOT", repo_root())
        .output()
        .expect("oclive-cli")
}

#[test]
fn explain_llm_error_prints_meaning() {
    let o = run_cli(&["explain", "LLM_ERROR"]);
    let stdout = String::from_utf8_lossy(&o.stdout);
    assert!(
        o.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&o.stderr)
    );
    assert!(stdout.contains("LLM") || stdout.contains("Meaning"));
}

#[test]
fn explain_unknown_code_fails() {
    let o = run_cli(&["explain", "UNKNOWN"]);
    assert!(!o.status.success());
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    );
    assert!(combined.to_ascii_lowercase().contains("unknown"));
}

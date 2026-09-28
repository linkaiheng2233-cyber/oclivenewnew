#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

fn run_cli(args: &[&str]) -> std::process::Output {
    common::cli_command()
        .args(args)
        .env("OCLIVE_ROOT", common::repo_root())
        .output()
        .expect("oclive-cli")
}

#[test]
fn explain_works_without_cargo_on_path() {
    let tools = tempfile::tempdir().expect("empty tools directory");
    let output = common::cli_command()
        .env("PATH", tools.path())
        .env("OCLIVE_ROOT", common::repo_root())
        .args(["explain", "LLM_ERROR"])
        .output()
        .expect("CLI must not require a Cargo launcher");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("LLM") || stdout.contains("Meaning"));
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

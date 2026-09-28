#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use tempfile::TempDir;

#[test]
fn init_dry_run_does_not_create_output_dir() {
    let td = TempDir::new().unwrap();
    let out = td.path().join("dry-run-kernel");
    let status = common::cli_command()
        .args([
            "init",
            "--dry-run",
            "--non-interactive",
            "--template",
            "robot-soul",
            "-o",
        ])
        .arg(&out)
        .status()
        .expect("oclive init --dry-run");
    assert!(status.success());
    assert!(!out.exists(), "dry-run must not create output directory");
    let o = common::cli_command()
        .args([
            "init",
            "--dry-run",
            "--non-interactive",
            "--template",
            "robot-soul",
            "-o",
        ])
        .arg(&out)
        .output()
        .expect("capture stdout");
    let stdout = String::from_utf8_lossy(&o.stdout);
    assert!(stdout.contains("dry-run") || stdout.contains("Directory structure"));
}

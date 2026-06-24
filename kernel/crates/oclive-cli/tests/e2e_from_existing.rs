#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

#[test]
fn from_existing_with_share_writes_oclive_share_toml() {
    let td = TempDir::new().unwrap();
    let mini = td.path().join("mini-kernel");
    fs::create_dir_all(mini.join("src")).unwrap();
    fs::write(
        mini.join("Cargo.toml"),
        r#"[package]
name = "mini-kernel"
version = "0.1.0"
edition = "2021"
[[bin]]
name = "mini-kernel"
path = "src/main.rs"
"#,
    )
    .unwrap();
    fs::write(mini.join("src/main.rs"), "fn main() {}").unwrap();
    fs::create_dir_all(mini.join("roles/default")).unwrap();
    fs::write(
        mini.join("roles/default/manifest.json"),
        r#"{"id":"default","name":"Default","version":"0.1.0"}"#,
    )
    .unwrap();
    fs::write(mini.join("roles/default/settings.json"), r#"{}"#).unwrap();

    let o = Command::new("cargo")
        .current_dir(repo_root())
        .args([
            "run",
            "-p",
            "oclive-cli",
            "--quiet",
            "--",
            "init",
            "--from-existing",
        ])
        .arg(&mini)
        .arg("--share")
        .arg("--json")
        .output()
        .expect("from-existing");
    assert!(
        o.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&o.stderr)
    );
    let share = mini.join(".oclive-share.toml");
    assert!(share.is_file(), "expected .oclive-share.toml after --share");
    let stdout = String::from_utf8_lossy(&o.stdout);
    assert!(stdout.contains("init") || stdout.contains("non_interactive"));
}

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

fn completion_output(shell: &str) -> String {
    let o = Command::new("cargo")
        .current_dir(repo_root())
        .args([
            "run",
            "-p",
            "oclive-cli",
            "--quiet",
            "--",
            "completions",
            shell,
        ])
        .output()
        .expect("completions");
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    String::from_utf8_lossy(&o.stdout).into_owned()
}

#[test]
fn bash_completion_mentions_init_and_bench() {
    let s = completion_output("bash");
    assert!(s.contains("init") || s.contains("_oclive"));
    assert!(s.contains("bench") || s.contains("Bench"));
}

#[test]
fn powershell_completion_non_empty() {
    let s = completion_output("powershell");
    assert!(s.len() > 40);
}

#[test]
fn zsh_and_fish_completion_non_empty() {
    for shell in ["zsh", "fish"] {
        let s = completion_output(shell);
        assert!(s.len() > 20, "{shell}");
    }
}

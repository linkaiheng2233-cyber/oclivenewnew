#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

fn completion_output(shell: &str) -> String {
    let o = common::cli_command()
        .args(["completions", shell])
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

//! registry 自动注册与 list。

use std::process::Command;

#[test]
fn init_registers_project_in_registry() {
    let home = tempfile::tempdir().expect("tempdir");
    let out = home.path().join("proj");
    let bin = env!("CARGO_BIN_EXE_oclive-cli");
    let st = Command::new(bin)
        .env("OCLIVE_HOME", home.path())
        .args([
            "init",
            "-o",
            out.to_str().expect("path"),
            "--non-interactive",
            "--preset",
            "minimal",
            "--project-type",
            "kernel-server",
            "--skip-role-pack",
            "--project-name",
            "reg-test-kernel",
            "--quiet",
        ])
        .status()
        .expect("spawn");
    assert!(st.success(), "init failed");
    let list = Command::new(bin)
        .env("OCLIVE_HOME", home.path())
        .args(["registry", "list", "--json"])
        .output()
        .expect("list");
    assert!(list.status.success());
    let body = String::from_utf8_lossy(&list.stdout);
    assert!(body.contains("reg-test-kernel"));
}

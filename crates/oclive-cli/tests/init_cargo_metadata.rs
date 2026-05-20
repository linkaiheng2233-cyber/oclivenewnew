//! `oclive init` Cargo 元数据参数。

use std::process::Command;

#[test]
fn init_writes_author_license_description() {
    let dir = tempfile::tempdir().expect("tempdir");
    let out = dir.path().join("proj");
    let bin = env!("CARGO_BIN_EXE_oclive-cli");
    let st = Command::new(bin)
        .args([
            "init",
            "-o",
            out.to_str().expect("path"),
            "--non-interactive",
            "--preset",
            "minimal",
            "--project-type",
            "kernel-server",
            "--author",
            "Keven",
            "--license",
            "MIT",
            "--description",
            "My AI chat kernel",
            "--skip-role-pack",
        ])
        .status()
        .expect("spawn");
    assert!(st.success(), "init failed");
    let cargo = std::fs::read_to_string(out.join("Cargo.toml")).expect("cargo");
    assert!(cargo.contains("authors = [\"Keven\"]"));
    assert!(cargo.contains("license = \"MIT\""));
    assert!(cargo.contains("description = \"My AI chat kernel\""));
}

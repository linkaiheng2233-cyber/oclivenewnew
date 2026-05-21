//! `oclive blueprint validate` 集成烟测。

use std::process::Command;

fn cli_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_oclive-cli"))
}

#[test]
fn blueprint_validate_valid_fixture() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/valid_blueprint.json"
    );
    let o = cli_bin()
        .args(["blueprint", "validate", path])
        .output()
        .unwrap();
    assert!(
        o.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&o.stderr)
    );
    let out = String::from_utf8_lossy(&o.stdout);
    assert!(out.contains("OK") || out.contains("valid"));
}

#[test]
fn blueprint_validate_invalid_fixture_exits_nonzero() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/invalid_blueprint.json"
    );
    let o = cli_bin()
        .args(["blueprint", "validate", path])
        .output()
        .unwrap();
    assert!(!o.status.success());
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(err.contains("llm") || err.contains("FAIL"), "stderr={err}");
}

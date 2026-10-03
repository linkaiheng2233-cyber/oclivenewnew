//! CLI wiring for the existing minimal role local-file preparation contract.

use std::{fs, process::Command};

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_oclive-cli"))
}

#[test]
fn caller_selected_minimal_file_and_asset_prepare_without_host_blueprint() {
    let pack = tempfile::tempdir().expect("pack root");
    fs::write(
        pack.path().join("content.json"),
        r#"{"persona_prompt":"A quiet guide.","visual_assets":["portrait.png"]}"#,
    )
    .expect("definition");
    fs::write(pack.path().join("portrait.png"), b"nonempty asset snapshot").expect("asset");

    let output = cli()
        .arg("pack")
        .arg("validate-minimal-local")
        .arg(pack.path())
        .arg("content.json")
        .output()
        .expect("cli invocation");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("1 assets"), "stdout={stdout}");
    assert!(stdout.contains("activation and media decoding not checked"));

    // The existing reference-Host validator remains a separate contract.
    let legacy = cli()
        .arg("pack")
        .arg("validate")
        .arg(pack.path())
        .output()
        .expect("reference pack validation");
    assert!(!legacy.status.success());
}

#[test]
fn missing_asset_and_escaping_definition_are_rejected() {
    let pack = tempfile::tempdir().expect("pack root");
    fs::write(
        pack.path().join("content.json"),
        r#"{"persona_prompt":"A quiet guide.","visual_assets":["missing.png"]}"#,
    )
    .expect("definition");

    for reference in ["content.json", "../content.json"] {
        let output = cli()
            .arg("pack")
            .arg("validate-minimal-local")
            .arg(pack.path())
            .arg(reference)
            .output()
            .expect("cli invocation");
        assert!(!output.status.success(), "reference={reference}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("minimal role"), "stderr={stderr}");
        assert!(!stderr.contains(&pack.path().display().to_string()));
    }
}

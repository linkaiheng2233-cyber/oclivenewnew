//! End-to-end: non-interactively generate a project and `cargo build` it.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;
use common::*;

use serde_json::Value;
use std::path::PathBuf;

#[test]
fn e2e_pack_create_validate_publish() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("com.example.demo");
    assert!(run_cli(&[
        "pack",
        "create",
        "-o",
        root.to_str().unwrap(),
        "--flat",
        "--id",
        "com.example.demo",
        "--name",
        "Demo",
        "--format-blueprint-v2",
    ])
    .success());
    assert!(root.join("pipeline.ocblueprint").exists());
    assert!(run_cli(&[
        "pack",
        "validate",
        root.to_str().unwrap(),
        "--host-version",
        "999.0.0",
    ])
    .success());
    let zip_path = tmp.path().join("out.oclivepack");
    assert!(run_cli(&[
        "pack",
        "publish",
        root.to_str().unwrap(),
        "-o",
        zip_path.to_str().unwrap(),
    ])
    .success());
    assert!(zip_path.is_file());
}

#[test]
fn e2e_pack_validate_robot_soul_example() {
    let example = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("examples")
        .join("robot-soul-minimal")
        .join("roles")
        .join("default");
    let example = example
        .canonicalize()
        .expect("robot-soul-minimal example path");
    assert!(
        example.join("manifest.json").is_file(),
        "missing {}",
        example.display()
    );
    let o = run_cli_output(&[
        "pack",
        "validate",
        example.to_str().unwrap(),
        "--host-version",
        "0.2.0",
        "--profile",
        "robot-soul",
    ]);
    assert!(
        o.status.success(),
        "robot-soul validate failed: {}",
        String::from_utf8_lossy(&o.stderr)
    );
}

#[test]
fn e2e_doctor_json_smoke() {
    let output = run_cli_output(&["doctor", "--json"]);
    let v: Value = serde_json::from_slice(&output.stdout).expect("doctor json");
    assert_eq!(v.get("schema_version").and_then(|x| x.as_u64()), Some(1));
    assert!(
        v.get("checks")
            .and_then(|x| x.as_array())
            .map(|a| !a.is_empty())
            == Some(true)
    );
}

#[test]
fn e2e_init_quick_non_interactive() {
    let tmp = tempfile::tempdir().unwrap();
    let out = tmp.path().join("quick");
    assert!(run_cli(&[
        "init",
        "--quick",
        "--non-interactive",
        "--quiet",
        "-o",
        out.to_str().unwrap(),
        "--project-name",
        "quick-chat",
    ])
    .success());
    assert!(out.join("Cargo.toml").is_file());
    assert!(!out.join("monolith.toml").exists());
    assert!(!out.join("roles").exists());
    let settings = out.join("CONFIG_REFERENCE.md");
    assert!(settings.is_file());
}

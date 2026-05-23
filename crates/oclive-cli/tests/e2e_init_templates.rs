//! 端到端：非交互生成项目并 `cargo build`。

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;
use common::*;

use serde_json::Value;
use std::fs;

#[test]
fn e2e_template_robot_soul_matches_manual_combo() {
    let tmp = tempfile::tempdir().unwrap();
    let out_tpl = tmp.path().join("tpl");
    let out_manual = tmp.path().join("manual");
    assert!(run_cli(&[
        "init",
        "--non-interactive",
        "--quiet",
        "--template",
        "robot-soul",
        "-o",
        out_tpl.to_str().unwrap(),
    ])
    .success());
    assert!(run_cli(&[
        "init",
        "--non-interactive",
        "--quiet",
        "--preset",
        "minimal",
        "--monolith",
        "-o",
        out_manual.to_str().unwrap(),
    ])
    .success());
    assert!(out_tpl.join("monolith.toml").is_file());
    assert!(out_manual.join("monolith.toml").is_file());
    let s_tpl = fs::read_to_string(out_tpl.join("roles/default/settings.json")).unwrap();
    let s_man = fs::read_to_string(out_manual.join("roles/default/settings.json")).unwrap();
    let v_tpl: Value = serde_json::from_str(&s_tpl).unwrap();
    let v_man: Value = serde_json::from_str(&s_man).unwrap();
    assert_eq!(v_tpl.get("plugin_backends"), v_man.get("plugin_backends"));
    assert!(out_tpl.join("roles/default/prompts/system.md").is_file());
    assert!(out_tpl.join("plugins/README.md").is_file());
    let m: Value = serde_json::from_str(
        &fs::read_to_string(out_tpl.join("roles/default/manifest.json")).unwrap(),
    )
    .unwrap();
    assert!(m.get("default_personality").is_some());
}

#[test]
fn e2e_template_headless_api_no_roles_by_default() {
    let tmp = tempfile::tempdir().unwrap();
    let out = tmp.path().join("api");
    assert!(run_cli(&[
        "init",
        "--non-interactive",
        "--quiet",
        "--template",
        "headless-api",
        "-o",
        out.to_str().unwrap(),
    ])
    .success());
    assert!(!out.join("roles").exists());
    assert!(!out.join("monolith.toml").exists());
    assert!(out.join("plugins/README.md").is_file());
}

#[test]
fn e2e_template_library_embed() {
    let tmp = tempfile::tempdir().unwrap();
    let out = tmp.path().join("lib");
    assert!(run_cli(&[
        "init",
        "--non-interactive",
        "--quiet",
        "--template",
        "library-embed",
        "-o",
        out.to_str().unwrap(),
    ])
    .success());
    assert!(!out.join("monolith.toml").exists());
    let cargo = fs::read_to_string(out.join("Cargo.toml")).unwrap();
    assert!(cargo.contains("[lib]") || cargo.contains("crate-type"));
    assert!(!out.join("roles").exists());
}

#[test]
fn e2e_template_robot_gateway_matches_manual_combo() {
    let tmp = tempfile::tempdir().unwrap();
    let out_tpl = tmp.path().join("tpl");
    let out_manual = tmp.path().join("manual");
    assert!(run_cli(&[
        "init",
        "--non-interactive",
        "--quiet",
        "--template",
        "robot-gateway",
        "-o",
        out_tpl.to_str().unwrap(),
    ])
    .success());
    assert!(run_cli(&[
        "init",
        "--non-interactive",
        "--quiet",
        "--preset",
        "mixed",
        "--monolith",
        "--skip-role-pack",
        "-o",
        out_manual.to_str().unwrap(),
    ])
    .success());
    assert!(out_tpl.join("monolith.toml").is_file());
    assert!(out_manual.join("monolith.toml").is_file());
    assert!(out_tpl.join("roles/gateway/settings.json").is_file());
    assert!(out_tpl.join("mcp_servers/README.md").is_file());
    assert!(!out_manual.join("roles").exists());
    let mt_tpl = fs::read_to_string(out_tpl.join("monolith.toml")).unwrap();
    let mt_man = fs::read_to_string(out_manual.join("monolith.toml")).unwrap();
    assert_eq!(mt_tpl, mt_man);
}

#[test]
fn e2e_template_dialogue_only_matches_manual_combo() {
    let tmp = tempfile::tempdir().unwrap();
    let out_tpl = tmp.path().join("tpl");
    let out_manual = tmp.path().join("manual");
    assert!(run_cli(&[
        "init",
        "--non-interactive",
        "--quiet",
        "--template",
        "dialogue-only",
        "-o",
        out_tpl.to_str().unwrap(),
    ])
    .success());
    assert!(run_cli(&[
        "init",
        "--non-interactive",
        "--quiet",
        "--preset",
        "full",
        "-o",
        out_manual.to_str().unwrap(),
    ])
    .success());
    assert!(!out_tpl.join("monolith.toml").exists());
    assert!(out_tpl.join("roles/default/settings.json").is_file());
    let s_tpl = fs::read_to_string(out_tpl.join("roles/default/settings.json")).unwrap();
    let s_man = fs::read_to_string(out_manual.join("roles/default/settings.json")).unwrap();
    let v_tpl: Value = serde_json::from_str(&s_tpl).unwrap();
    let v_man: Value = serde_json::from_str(&s_man).unwrap();
    assert_eq!(v_tpl.get("plugin_backends"), v_man.get("plugin_backends"));
    assert!(out_tpl.join("docs/BLUEPRINT_V2_POINTER.md").is_file());
}

#[test]
fn e2e_list_templates_prints_matrix() {
    let output = run_cli_output(&["init", "--list-templates"]);
    assert!(output.status.success());
    let out = String::from_utf8_lossy(&output.stdout);
    assert!(out.contains("robot-gateway"));
    assert!(out.contains("dialogue-only"));
    assert!(out.contains("preset"));
}

#[test]
fn e2e_template_robot_gateway_has_mcp_scaffold() {
    let tmp = tempfile::tempdir().unwrap();
    let out = tmp.path().join("gw");
    assert!(run_cli(&[
        "init",
        "--non-interactive",
        "--quiet",
        "--template",
        "robot-gateway",
        "-o",
        out.to_str().unwrap(),
    ])
    .success());
    assert!(out.join("mcp_servers/README.md").is_file());
    assert!(out.join("mcp_servers/smart_home.example.json").is_file());
    let settings = fs::read_to_string(out.join("roles/gateway/settings.json")).unwrap();
    assert!(settings.contains("\"agent\""));
    assert!(settings.contains("agent_mcp"));
    let v: Value = serde_json::from_str(&settings).unwrap();
    assert_eq!(
        v.get("plugin_backends")
            .and_then(|p| p.get("agent"))
            .and_then(|a| a.as_str()),
        Some("builtin")
    );
    assert!(out.join("docs/WELD_BENCH_REPORT.md").is_file());
}

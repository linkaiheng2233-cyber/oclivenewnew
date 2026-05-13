//! P2-5：权限模型边界补充（高危组合 consent、开发者模式扫描根、桥接权限收集）。

use oclive_kernel_runtime::domain::permission_tokens::bridge_permission_tokens_from_manifest;
use oclive_kernel_runtime::domain::plugin_install_consent::ensure_accepted_permissions_subset_declared;
use oclive_kernel_runtime::infrastructure::directory_plugins::{
    plugin_scan_container_roots, HostPluginsFile, OclivePluginManifest,
};
use std::fs;

#[test]
fn plugin_scan_extra_roots_require_developer_effective() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let roles = tmp.path().join("roles");
    let app = tmp.path().join("appdata");
    fs::create_dir_all(roles.join("r")).expect("roles");
    fs::create_dir_all(app.join("plugins")).expect("plugins");
    let extra = tmp.path().join("extra_market_plugins");
    fs::create_dir_all(&extra).expect("extra");
    let extra_s = extra.to_string_lossy().into_owned();

    let host_off = HostPluginsFile {
        developer_mode: false,
        extra_plugin_roots: vec![extra_s.clone()],
        ..Default::default()
    };
    let roots_off = plugin_scan_container_roots(&roles, &app, &host_off);
    assert!(
        !roots_off.iter().any(|p| p == &extra),
        "extra roots must be ignored when developer_mode is off"
    );

    let host_on = HostPluginsFile {
        developer_mode: true,
        extra_plugin_roots: vec![extra_s],
        ..Default::default()
    };
    let roots_on = plugin_scan_container_roots(&roles, &app, &host_on);
    assert!(
        roots_on.iter().any(|p| p == &extra),
        "extra roots must be scanned when developer_mode is on"
    );
}

#[test]
fn bridge_permission_collects_high_risk_invoke_tokens() {
    let raw = r#"{
        "schema_version": 1,
        "id": "p2b5_shell",
        "version": "1.0.0",
        "shell": {
            "entry": "ui/index.html",
            "bridge": {
                "invoke": ["rpc:invoke", "process:spawn", "network:*"]
            }
        }
    }"#;
    let m: OclivePluginManifest = serde_json::from_str(raw).expect("manifest");
    let toks = bridge_permission_tokens_from_manifest(&m);
    assert!(toks.contains(&"rpc:invoke".to_string()));
    assert!(toks.contains(&"process:spawn".to_string()));
    assert!(toks.contains(&"network:*".to_string()));
}

#[test]
fn install_consent_accepts_declared_high_risk_combo_rejects_unknown() {
    let declared = vec![
        "process:spawn".into(),
        "network:*".into(),
        "rpc:invoke".into(),
    ];
    let accepted = vec![
        "process:spawn".into(),
        "network:*".into(),
        "rpc:invoke".into(),
    ];
    ensure_accepted_permissions_subset_declared(&declared, &accepted).expect("subset ok");
    let bad = vec!["filesystem:*".into()];
    assert!(
        ensure_accepted_permissions_subset_declared(&declared, &bad).is_err(),
        "unknown token must not be accepted"
    );
}

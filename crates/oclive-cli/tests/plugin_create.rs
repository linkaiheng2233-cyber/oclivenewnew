//! `oclive plugin create` 脚手架与 manifest 校验。

use std::process::Command;

#[test]
fn plugin_create_directory_llm_validates() {
    let dir = tempfile::tempdir().expect("tempdir");
    let out = dir.path().join("out").join("plugin");
    let bin = env!("CARGO_BIN_EXE_oclive-cli");
    let st = Command::new(bin)
        .args([
            "plugin",
            "create",
            "test-plugin",
            "--type",
            "directory",
            "--provides",
            "llm",
            "-o",
            out.to_str().expect("path"),
            "--non-interactive",
        ])
        .status()
        .expect("spawn");
    assert!(st.success(), "plugin create failed");
    let manifest_path = out.join("com.oclive.plugin.test-plugin/manifest.json");
    assert!(manifest_path.is_file(), "manifest missing");
    let raw = std::fs::read_to_string(&manifest_path).expect("read manifest");
    oclive_validation::validate_directory_plugin_manifest_permissions(&raw)
        .expect("permissions validate");
    assert!(out.join("com.oclive.plugin.test-plugin/rpc_server.mjs").is_file());
}

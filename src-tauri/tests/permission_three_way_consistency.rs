//! 目录插件 permissions：manifest ↔ oclive_validation ↔ 运行时 grant 三面一致。

use oclive_validation::{
    manifest_declares_process_spawn, validate_directory_plugin_manifest_permissions,
    validate_permissions_list, NETWORK_GRANT_REMOTE_PLUGIN, PROCESS_SPAWN,
};
use oclivenewnew_tauri::infrastructure::directory_plugins::OclivePluginManifest;
use oclivenewnew_tauri::infrastructure::high_risk_grants::HighRiskGrantStore;
use oclivenewnew_tauri::infrastructure::remote_plugin::{
    RemoteMemoryRetrievalHttp, RemotePluginHttpConfig,
};
use std::fs;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;
use tempfile::tempdir;

fn write_manifest(dir: &std::path::Path, json: &str) {
    fs::write(dir.join("manifest.json"), json).unwrap();
}

#[test]
fn validation_rejects_unknown_permission() {
    let json = r#"{"permissions":["process"]}"#;
    assert!(validate_directory_plugin_manifest_permissions(json).is_err());
}

#[test]
fn validation_accepts_missing_and_empty_permissions() {
    validate_permissions_list(&[]).unwrap();
    let json = r#"{"schema_version":1,"id":"x","version":"1.0.0"}"#;
    validate_directory_plugin_manifest_permissions(json).unwrap();
    let json2 = r#"{"permissions":[]}"#;
    validate_directory_plugin_manifest_permissions(json2).unwrap();
}

#[test]
fn manifest_load_validates_permissions() {
    let dir = tempdir().unwrap();
    write_manifest(
        dir.path(),
        r#"{
          "schema_version": 1,
          "id": "bad.perms",
          "version": "1.0.0",
          "permissions": ["filesystem"]
        }"#,
    );
    assert!(OclivePluginManifest::load_from_dir(dir.path()).is_err());
}

#[test]
fn process_spawn_requires_grant_when_declared() {
    let dir = tempdir().unwrap();
    let app = dir.path().join("app_data");
    fs::create_dir_all(&app).unwrap();
    let grants = HighRiskGrantStore::load(app, true);

    write_manifest(
        dir.path(),
        r#"{
          "schema_version": 1,
          "id": "plug.spawn",
          "version": "1.0.0",
          "permissions": ["process:spawn"],
          "process": { "command": "node", "args": ["x.js"] }
        }"#,
    );
    let manifest = OclivePluginManifest::load_from_dir(dir.path()).unwrap();
    assert!(manifest_declares_process_spawn(&manifest.permissions, true));
    assert!(!grants.is_process_spawn_granted("plug.spawn"));

    grants.grant_process_spawn("plug.spawn").unwrap();
    assert!(grants.is_process_spawn_granted("plug.spawn"));
}

#[test]
fn legacy_manifest_without_permissions_still_declares_spawn_with_process() {
    let json = r#"{
      "schema_version": 1,
      "id": "legacy.plug",
      "version": "1.0.0",
      "process": { "command": "node", "args": ["x.js"] }
    }"#;
    validate_directory_plugin_manifest_permissions(json).unwrap();
    let dir = tempdir().unwrap();
    write_manifest(dir.path(), json);
    let manifest = OclivePluginManifest::load_from_dir(dir.path()).unwrap();
    assert!(manifest_declares_process_spawn(
        &manifest.permissions,
        manifest.process.is_some()
    ));
}

#[test]
fn explicit_permissions_without_process_spawn_blocks_spawn_declaration() {
    let perms = vec!["network:*".to_string()];
    assert!(!manifest_declares_process_spawn(&perms, true));
}

#[test]
fn remote_http_requires_network_grant() {
    let dir = tempdir().unwrap();
    let grants = HighRiskGrantStore::load(dir.path().to_path_buf(), true);
    let fb = Arc::new(AtomicBool::new(true));
    let cfg = RemotePluginHttpConfig {
        endpoint: "http://127.0.0.1:9".into(),
        bearer_token: None,
        timeout: Duration::from_millis(100),
    };
    let client = RemoteMemoryRetrievalHttp::new(
        cfg,
        fb,
        grants.clone(),
        Some(NETWORK_GRANT_REMOTE_PLUGIN.to_string()),
    )
    .unwrap();
    use oclivenewnew_tauri::domain::memory_retrieval::{MemoryRetrieval, MemoryRetrievalInput};
    use oclivenewnew_tauri::error::AppError;

    let err = client
        .rank_memories(MemoryRetrievalInput {
            memories: &[],
            user_query: "hi",
            scene_id: None,
            limit: 3,
        })
        .unwrap_err();
    assert!(matches!(err, AppError::HighRiskCapabilityNotGranted { .. }));

    grants.grant_network(NETWORK_GRANT_REMOTE_PLUGIN).unwrap();
    assert!(grants.require_network(NETWORK_GRANT_REMOTE_PLUGIN).is_ok());
    let after_grant = client.rank_memories(MemoryRetrievalInput {
        memories: &[],
        user_query: "hi",
        scene_id: None,
        limit: 3,
    });
    if let Err(e) = after_grant {
        assert!(
            !matches!(e, AppError::HighRiskCapabilityNotGranted { .. }),
            "expected network grant to unblock HTTP, got {:?}",
            e
        );
    }
}

#[test]
fn grant_file_serializes_permission_spec_keys() {
    let dir = tempdir().unwrap();
    let grants = HighRiskGrantStore::load(dir.path().to_path_buf(), false);
    grants.grant_process_spawn("p1").unwrap();
    let raw = fs::read_to_string(dir.path().join("high_risk_grants.json")).unwrap();
    assert!(raw.contains("\"process:spawn\""));
    assert!(raw.contains("p1"));
    assert!(!raw.contains("directory_plugin_process_spawn"));
}

#[test]
fn allowed_permission_tokens_match_spec() {
    assert!(validate_permissions_list(&[PROCESS_SPAWN.into()]).is_ok());
}

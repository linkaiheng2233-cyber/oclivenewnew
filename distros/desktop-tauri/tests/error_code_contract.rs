//! Hot-path structured error codes: kernel JSON body + manifest validation surface.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use oclive_kernel_host::infrastructure::directory_plugins::OclivePluginManifest;
use oclive_kernel_host::infrastructure::MockLlmClient;
use oclive_kernel_host::state::AppStateBuilder;
use oclive_kernel_types::error::{AppError, KernelErrorBody};
use std::sync::{Arc, Mutex};
use tempfile::TempDir;

static MIGRATION_ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn plugin_manifest_invalid_maps_to_structured_code() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("manifest.json"), r#"{"schema_version":1}"#).unwrap();
    let err = OclivePluginManifest::load_from_dir(dir.path()).unwrap_err();
    let app = AppError::PluginManifestInvalid(err.to_string());
    assert_eq!(app.code(), "PLUGIN_MANIFEST_INVALID");
    let body: KernelErrorBody = serde_json::from_str(&app.to_kernel_json()).expect("kernel json");
    assert_eq!(body.code, "PLUGIN_MANIFEST_INVALID");
}

#[tokio::test]
async fn db_migration_failed_maps_to_structured_code_and_marker() {
    let tmp = TempDir::new().unwrap();
    let mig_dir = tmp.path().join("migrations");
    std::fs::create_dir_all(&mig_dir).unwrap();
    std::fs::write(mig_dir.join("001_bad.sql"), "NOT VALID SQL;").unwrap();
    let app_data = tmp.path().join("appdata");
    std::fs::create_dir_all(&app_data).unwrap();
    {
        let _guard = MIGRATION_ENV_LOCK.lock().expect("env lock");
        std::env::set_var("OCLIVE_MIGRATIONS_DIR", mig_dir.to_string_lossy().as_ref());
    }

    let fixture_packs = tmp.path().join("pack_fixture");
    std::fs::create_dir_all(&fixture_packs).unwrap();
    let llm = Arc::new(MockLlmClient {
        reply: "ok".to_string(),
    });
    let err = match AppStateBuilder::in_memory_test(llm, &fixture_packs, None)
        .with_app_data_dir(&app_data)
        .build()
        .await
    {
        Ok(_) => panic!("invalid migration SQL must fail bootstrap"),
        Err(e) => e,
    };

    assert_eq!(err.code(), "DB_MIGRATION_FAILED");
    let body: KernelErrorBody = serde_json::from_str(&err.to_kernel_json()).expect("kernel json");
    assert_eq!(body.code, "DB_MIGRATION_FAILED");
    assert!(
        app_data.join("migration_failed.json").is_file(),
        "migration failure must write migration_failed.json marker"
    );
    {
        let _guard = MIGRATION_ENV_LOCK.lock().expect("env lock");
        std::env::remove_var("OCLIVE_MIGRATIONS_DIR");
    }
}

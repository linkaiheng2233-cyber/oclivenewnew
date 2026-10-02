//! An isolated integration-test process checks LLM settings after reopening a migrated file DB.

use oclive_kernel_host::domain::user_llm_env::{KEY_LLM_PROVIDER, KEY_OLLAMA_BASE};
use oclive_kernel_host::infrastructure::MockLlmClient;
use oclive_kernel_host::service::llm_settings::reload_llm_user_env_impl;
use oclive_kernel_host::state::AppStateBuilder;
use std::ffi::OsString;
use std::sync::Arc;

const ENV_KEYS: &[&str] = &[
    "OCLIVE_APP_DATA",
    "OLLAMA_BASE_URL",
    "OCLIVE_REMOTE_LLM_URL",
    "OCLIVE_REMOTE_LLM_TOKEN",
    "OCLIVE_LLM_CLOUD_API_STYLE",
    "OCLIVE_LOCAL_LLM_MODEL_PATH",
    "OCLIVE_LOCAL_LLM_LORA_PATH",
    "OCLIVE_LLM_BACKEND",
];

struct RestoreEnvironment(Vec<(&'static str, Option<OsString>)>);

impl RestoreEnvironment {
    fn capture() -> Self {
        Self(
            ENV_KEYS
                .iter()
                .map(|&key| (key, std::env::var_os(key)))
                .collect(),
        )
    }
}

impl Drop for RestoreEnvironment {
    fn drop(&mut self) {
        for (key, value) in &self.0 {
            match value {
                Some(value) => std::env::set_var(key, value),
                None => std::env::remove_var(key),
            }
        }
    }
}

#[tokio::test]
async fn migrated_file_db_settings_apply_after_app_state_rebuild() {
    let _environment = RestoreEnvironment::capture();
    let temp = tempfile::tempdir().expect("isolated app data");
    let app_data = temp.path().join("app-data");
    let roles = temp.path().join("role-fixtures");
    std::fs::create_dir_all(&app_data).expect("app data root");
    std::fs::create_dir_all(&roles).expect("isolated roles root");
    std::env::set_var("OCLIVE_APP_DATA", &app_data);
    let db_path = app_data.join("app.db");
    let llm = Arc::new(MockLlmClient {
        reply: "unused".to_owned(),
    });

    let first = AppStateBuilder::production(&db_path, roles.clone(), &app_data)
        .with_llm_client(llm.clone())
        .build()
        .await
        .expect("first production state with migrated file DB");
    first
        .db_manager
        .upsert_app_setting(KEY_LLM_PROVIDER, "local")
        .await
        .expect("store local provider");
    first
        .db_manager
        .upsert_app_setting(KEY_OLLAMA_BASE, "http://127.0.0.1:77777")
        .await
        .expect("store file-backed base URL");
    assert_eq!(
        reload_llm_user_env_impl(&first)
            .await
            .expect("apply first state setting"),
        "local"
    );
    first.db_manager.close_pool().await;
    drop(first);

    std::env::set_var("OLLAMA_BASE_URL", "http://127.0.0.1:11111");
    let second = AppStateBuilder::production(&db_path, roles, &app_data)
        .with_llm_client(llm)
        .build()
        .await
        .expect("rebuild production state from same file DB");
    assert_eq!(
        second
            .db_manager
            .get_app_setting(KEY_OLLAMA_BASE)
            .await
            .expect("read persisted URL")
            .as_deref(),
        Some("http://127.0.0.1:77777")
    );
    assert_eq!(
        reload_llm_user_env_impl(&second)
            .await
            .expect("apply reopened file DB setting"),
        "local"
    );
    assert_eq!(
        std::env::var("OLLAMA_BASE_URL").as_deref(),
        Ok("http://127.0.0.1:77777")
    );
    assert_eq!(std::env::var("OCLIVE_LLM_BACKEND").as_deref(), Ok("ollama"));
    second.db_manager.close_pool().await;
}

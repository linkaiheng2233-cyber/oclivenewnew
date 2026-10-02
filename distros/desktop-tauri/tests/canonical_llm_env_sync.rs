//! A separate integration-test process owns the canonical path and LLM environment.

use oclive_kernel_host::domain::user_llm_env::{KEY_LLM_PROVIDER, KEY_OLLAMA_BASE};
use oclive_kernel_host::infrastructure::MockLlmClient;
use oclive_kernel_host::service::llm_settings::reload_llm_user_env_impl;
use oclive_kernel_host::state::AppState;
use oclivenewnew_tauri::api::llm_settings::seed_shell_llm_from_canonical;
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
async fn canonical_seed_marks_and_applies_shell_llm_environment() {
    let _environment = RestoreEnvironment::capture();
    let temp = tempfile::tempdir().expect("isolated app data");
    let app_data = temp.path().join("canonical");
    let roles = temp.path().join("role-fixtures");
    std::fs::create_dir_all(&app_data).expect("canonical app data");
    std::fs::create_dir_all(&roles).expect("isolated roles");
    std::env::set_var("OCLIVE_APP_DATA", &app_data);

    let db_path = app_data.join("app.db");
    let url = format!("sqlite:{}?mode=rwc", db_path.display());
    let pool = sqlx::SqlitePool::connect(&url)
        .await
        .expect("canonical SQLite file");
    sqlx::query("CREATE TABLE app_settings (key TEXT PRIMARY KEY NOT NULL, value TEXT NOT NULL)")
        .execute(&pool)
        .await
        .expect("canonical settings table");
    for (key, value) in [
        (KEY_LLM_PROVIDER, "local"),
        (KEY_OLLAMA_BASE, "http://127.0.0.1:22222"),
    ] {
        sqlx::query("INSERT INTO app_settings (key, value) VALUES (?, ?)")
            .bind(key)
            .bind(value)
            .execute(&pool)
            .await
            .expect("canonical setting");
    }
    pool.close().await;

    let state = AppState::new_in_memory_with_llm(
        Arc::new(MockLlmClient {
            reply: "unused".to_owned(),
        }),
        &roles,
    )
    .await
    .expect("isolated UI shell");
    state
        .db_manager
        .upsert_app_setting(KEY_LLM_PROVIDER, "local")
        .await
        .expect("old shell provider");
    state
        .db_manager
        .upsert_app_setting(KEY_OLLAMA_BASE, "http://127.0.0.1:11111")
        .await
        .expect("old shell URL");
    reload_llm_user_env_impl(&state)
        .await
        .expect("apply old shell URL");
    assert_eq!(
        std::env::var("OLLAMA_BASE_URL").as_deref(),
        Ok("http://127.0.0.1:11111")
    );

    seed_shell_llm_from_canonical(&state).await;
    assert_eq!(
        state
            .db_manager
            .get_app_setting(KEY_OLLAMA_BASE)
            .await
            .expect("read seeded shell URL")
            .as_deref(),
        Some("http://127.0.0.1:22222")
    );
    assert_eq!(
        std::env::var("OLLAMA_BASE_URL").as_deref(),
        Ok("http://127.0.0.1:22222")
    );
    assert_eq!(std::env::var("OCLIVE_LLM_BACKEND").as_deref(), Ok("ollama"));
}

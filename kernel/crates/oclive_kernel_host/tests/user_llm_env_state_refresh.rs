//! The integration-test binary isolates process-wide LLM environment variables.

use oclive_kernel_host::domain::user_llm_env::{
    apply_user_llm_env, KEY_LLM_PROVIDER, KEY_OLLAMA_BASE,
};
use oclive_kernel_host::infrastructure::MockLlmClient;
use oclive_kernel_host::service::llm_settings::reload_llm_user_env_impl;
use oclive_kernel_host::state::AppState;
use std::ffi::OsString;
use std::sync::Arc;
use tempfile::TempDir;

const ENV_KEYS: &[&str] = &[
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
async fn actual_state_refreshes_only_after_a_dirty_mark() {
    let _environment = RestoreEnvironment::capture();
    let temp = TempDir::new().expect("isolated role root");
    let state = AppState::new_in_memory_with_llm(
        Arc::new(MockLlmClient {
            reply: "unused".to_owned(),
        }),
        temp.path(),
    )
    .await
    .expect("in-memory AppState");

    state
        .db_manager
        .upsert_app_setting(KEY_LLM_PROVIDER, "local")
        .await
        .expect("store provider");
    state
        .db_manager
        .upsert_app_setting(KEY_OLLAMA_BASE, "http://127.0.0.1:11111")
        .await
        .expect("store old base URL");
    assert_eq!(
        reload_llm_user_env_impl(&state)
            .await
            .expect("reload old setting"),
        "local"
    );
    assert_eq!(
        std::env::var("OLLAMA_BASE_URL").as_deref(),
        Ok("http://127.0.0.1:11111")
    );
    assert_eq!(std::env::var("OCLIVE_LLM_BACKEND").as_deref(), Ok("ollama"));

    state
        .db_manager
        .upsert_app_setting(KEY_OLLAMA_BASE, "http://127.0.0.1:22222")
        .await
        .expect("store new base URL");
    apply_user_llm_env(&state)
        .await
        .expect("unchanged version uses the clean fast path");
    assert_eq!(
        std::env::var("OLLAMA_BASE_URL").as_deref(),
        Ok("http://127.0.0.1:11111"),
        "direct DB writes do not implicitly invalidate the applied version"
    );

    state.mark_user_llm_env_dirty();
    apply_user_llm_env(&state)
        .await
        .expect("apply marked setting");
    assert_eq!(
        std::env::var("OLLAMA_BASE_URL").as_deref(),
        Ok("http://127.0.0.1:22222")
    );
    assert_eq!(std::env::var("OCLIVE_LLM_BACKEND").as_deref(), Ok("ollama"));
}

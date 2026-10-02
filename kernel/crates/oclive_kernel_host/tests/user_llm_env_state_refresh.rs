//! The integration-test binary isolates process-wide LLM environment variables.

use oclive_kernel_host::domain::chat_engine::process_message;
use oclive_kernel_host::domain::user_llm_env::{
    apply_user_llm_env, KEY_LLM_PROVIDER, KEY_OLLAMA_BASE,
};
use oclive_kernel_host::infrastructure::MockLlmClient;
use oclive_kernel_host::service::llm_settings::{
    reload_llm_user_env_impl, save_llm_user_settings_impl, SaveLlmUserSettingsRequest,
};
use oclive_kernel_host::state::AppState;
use oclive_kernel_types::models::dto::SendMessageRequest;
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
async fn app_state_reload_save_and_chat_apply_expected_llm_environment() {
    let _environment = RestoreEnvironment::capture();
    let temp = TempDir::new().expect("isolated role root");
    let role_dir = temp.path().join("env-probe");
    std::fs::create_dir(&role_dir).expect("role directory");
    std::fs::write(
        role_dir.join("manifest.json"),
        r#"{"id":"env-probe","name":"Env Probe","version":"1","author":"test","description":"isolated settings fixture","default_personality":[0.5,0.5,0.5,0.5,0.5,0.5,0.5],"evolution":{},"user_relations":{"friend":{"prompt_hint":"test"}},"default_relation":"friend","memory_config":{"scene_weight_multiplier":1.0,"topic_weights":{}}}"#,
    )
    .expect("role manifest");
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

    save_llm_user_settings_impl(
        &state,
        &SaveLlmUserSettingsRequest {
            role_id: "env-probe".to_owned(),
            session_id: None,
            provider: "local".to_owned(),
            cloud_vendor: None,
            cloud_api_style: None,
            ollama_base_url: Some("http://127.0.0.1:33333".to_owned()),
            local_models_dir: None,
            local_model_path: None,
            adult_content_acknowledged: false,
            ollama_model: None,
            remote_url: None,
            remote_token: None,
            remote_model: None,
        },
    )
    .await
    .expect("production save path refreshes the LLM environment");
    assert_eq!(
        std::env::var("OLLAMA_BASE_URL").as_deref(),
        Ok("http://127.0.0.1:33333")
    );
    assert_eq!(
        state
            .db_manager
            .get_app_setting(KEY_OLLAMA_BASE)
            .await
            .expect("read saved base URL")
            .as_deref(),
        Some("http://127.0.0.1:33333")
    );

    state
        .db_manager
        .upsert_app_setting(KEY_OLLAMA_BASE, "http://127.0.0.1:44444")
        .await
        .expect("store pending chat setting");
    state.mark_user_llm_env_dirty();
    let response = process_message(
        &state,
        &SendMessageRequest {
            role_id: "env-probe".to_owned(),
            user_message: "hello".to_owned(),
            ..Default::default()
        },
    )
    .await
    .expect("chat entry applies pending LLM environment");
    assert_eq!(response.reply, "unused");
    assert_eq!(
        std::env::var("OLLAMA_BASE_URL").as_deref(),
        Ok("http://127.0.0.1:44444")
    );
}

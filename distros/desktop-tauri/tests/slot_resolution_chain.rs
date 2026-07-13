//! D-ARCH-01：六槽有效 backends 解析链（legacy / v2 + 内存 session override + env LLM + host ceiling）。

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use oclive_kernel_host::domain::host_profile::HostProfile;
use oclive_kernel_host::infrastructure::MockLlmClient;
use oclive_kernel_host::models::plugin_backends::{
    AgentBackend, EmotionBackend, EventBackend, LlmBackend, MemoryBackend, PluginBackends,
    PromptBackend,
};
use oclive_kernel_host::state::{AppState, AppStateBuilder};
use oclive_kernel_types::models::dto::{
    GetPluginResolutionDebugRequest, SetSessionPluginBackendRequest,
};
use oclive_kernel_types::models::PluginBackendSource;
use oclivenewnew_tauri::api::role::{
    get_plugin_resolution_debug_impl, load_role_impl, set_session_plugin_backend_impl,
};
use std::fs;
use std::sync::{Arc, Mutex};
use tempfile::TempDir;

static ENV_TEST_LOCK: Mutex<()> = Mutex::new(());

fn write_legacy_role(roles_root: &TempDir, role_id: &str, memory: &str, llm: &str) {
    let role_dir = roles_root.path().join(role_id);
    fs::create_dir_all(role_dir.join("scenes/default")).unwrap();
    fs::write(
        role_dir.join("manifest.json"),
        format!(
            r#"{{"id":"{role_id}","name":"Fixture","version":"0.1.0","author":"t","description":"t","scenes":["default"],"default_personality":[0.5,0.5,0.5,0.5,0.5,0.5,0.5],"user_relations":{{"friend":{{"initial_favorability":50.0,"favor_multiplier":1.0}}}},"default_relation":"friend"}}"#
        ),
    )
    .unwrap();
    fs::write(role_dir.join("core_personality.txt"), "test persona").unwrap();
    fs::write(
        role_dir.join("settings.json"),
        format!(
            r#"{{"plugin_backends":{{"memory":"{memory}","llm":"{llm}","emotion":"remote","event":"remote","prompt":"remote","agent":"remote"}}}}"#
        ),
    )
    .unwrap();
    fs::write(
        role_dir.join("scenes/default/scene.json"),
        r#"{"id":"default","label":"default"}"#,
    )
    .unwrap();
}

fn vscode_ceiling_host() -> HostProfile {
    HostProfile {
        distro_id: "vscode-test".into(),
        skip_agent: true,
        backends_ceiling: Some(PluginBackends {
            memory: MemoryBackend::Builtin,
            emotion: EmotionBackend::Builtin,
            event: EventBackend::Builtin,
            prompt: PromptBackend::Builtin,
            llm: LlmBackend::Ollama,
            agent: AgentBackend::None,
            ..Default::default()
        }),
        ..HostProfile::default()
    }
}

#[tokio::test]
async fn v2_pack_session_memory_override_in_memory_only() {
    let llm = Arc::new(MockLlmClient {
        reply: "ok".to_string(),
    });
    let state = AppState::new_in_memory_with_llm(llm, common::roles_dir())
        .await
        .expect("state");
    load_role_impl(&state, "mumu", true).await.expect("load");

    set_session_plugin_backend_impl(
        &state,
        &SetSessionPluginBackendRequest {
            role_id: "mumu".to_string(),
            module: "memory".to_string(),
            backend: Some(Some("remote".to_string())),
            local_memory_provider_id: None,
            session_id: Some("arch-chain".to_string()),
        },
    )
    .await
    .expect("override");

    let debug = get_plugin_resolution_debug_impl(
        &state,
        &GetPluginResolutionDebugRequest {
            role_id: "mumu".to_string(),
            session_id: Some("arch-chain".to_string()),
        },
    )
    .await
    .expect("debug");

    assert!(debug.session_namespace.contains("arch-chain"));
    assert_eq!(
        debug.plugin_backends_effective.memory,
        MemoryBackend::Remote
    );
    assert_eq!(
        debug.plugin_backends_effective_sources.memory,
        PluginBackendSource::SessionOverride
    );
}

#[tokio::test]
async fn legacy_plugin_backends_pack_resolves_without_slot_registry() {
    let _guard = ENV_TEST_LOCK.lock().expect("env lock");
    std::env::remove_var("OCLIVE_LLM_BACKEND");

    let tmp = TempDir::new().unwrap();
    write_legacy_role(&tmp, "legacy_slot", "builtin", "ollama");
    let llm = Arc::new(MockLlmClient {
        reply: "ok".to_string(),
    });
    let state = AppState::new_in_memory_with_llm(llm, tmp.path())
        .await
        .expect("state");
    load_role_impl(&state, "legacy_slot", true)
        .await
        .expect("load");

    let debug = get_plugin_resolution_debug_impl(
        &state,
        &GetPluginResolutionDebugRequest {
            role_id: "legacy_slot".to_string(),
            session_id: None,
        },
    )
    .await
    .expect("debug");

    assert_eq!(
        debug.plugin_backends_effective.memory,
        MemoryBackend::Builtin
    );
    assert_eq!(debug.plugin_backends_effective.llm, LlmBackend::Ollama);
    assert!(debug.llm_env_override.is_none());
}

#[tokio::test]
async fn env_llm_override_surfaces_in_debug_chain() {
    let _guard = ENV_TEST_LOCK.lock().expect("env lock");
    std::env::remove_var("OCLIVE_LLM_BACKEND");

    let llm = Arc::new(MockLlmClient {
        reply: "ok".to_string(),
    });
    let state = AppState::new_in_memory_with_llm(llm, common::roles_dir())
        .await
        .expect("state");
    load_role_impl(&state, "mumu", true).await.expect("load");
    std::env::set_var("OCLIVE_LLM_BACKEND", "remote");

    let debug = get_plugin_resolution_debug_impl(
        &state,
        &GetPluginResolutionDebugRequest {
            role_id: "mumu".to_string(),
            session_id: None,
        },
    )
    .await
    .expect("debug");

    assert_eq!(debug.llm_env_override.as_deref(), Some("remote"));
    std::env::remove_var("OCLIVE_LLM_BACKEND");
}

#[tokio::test]
async fn host_profile_ceiling_replaces_pack_remote_before_session_override() {
    let tmp = TempDir::new().unwrap();
    write_legacy_role(&tmp, "ceiling_role", "remote", "remote");
    let llm: Arc<dyn oclive_kernel_host::infrastructure::llm::LlmClient> =
        Arc::new(MockLlmClient {
            reply: "ok".to_string(),
        });

    let open_state = AppState::new_in_memory_with_llm(Arc::clone(&llm), tmp.path())
        .await
        .expect("state");
    load_role_impl(&open_state, "ceiling_role", true)
        .await
        .expect("load");
    let open = get_plugin_resolution_debug_impl(
        &open_state,
        &GetPluginResolutionDebugRequest {
            role_id: "ceiling_role".to_string(),
            session_id: None,
        },
    )
    .await
    .expect("debug");
    assert_eq!(open.plugin_backends_effective.memory, MemoryBackend::Remote);
    assert_eq!(open.plugin_backends_effective.llm, LlmBackend::Remote);
    assert_eq!(
        open.plugin_backends_effective_sources.memory,
        PluginBackendSource::PackDefault
    );

    let ceiling_state = AppStateBuilder::in_memory_test(Arc::clone(&llm), tmp.path(), None)
        .with_host_profile(vscode_ceiling_host())
        .build()
        .await
        .expect("state");
    load_role_impl(&ceiling_state, "ceiling_role", true)
        .await
        .expect("load");
    let capped = get_plugin_resolution_debug_impl(
        &ceiling_state,
        &GetPluginResolutionDebugRequest {
            role_id: "ceiling_role".to_string(),
            session_id: None,
        },
    )
    .await
    .expect("debug");
    assert_eq!(
        capped.plugin_backends_effective.memory,
        MemoryBackend::Builtin
    );
    assert_eq!(capped.plugin_backends_effective.llm, LlmBackend::Ollama);
    assert_eq!(
        capped.plugin_backends_effective.emotion,
        EmotionBackend::Builtin
    );
    assert_eq!(
        capped.plugin_backends_effective_sources.memory,
        PluginBackendSource::PackDefault,
        "ceiling merge does not change source map; effective backends reflect profile_override"
    );

    let ceiling_v2_state =
        AppStateBuilder::in_memory_test(Arc::clone(&llm), common::roles_dir(), None)
            .with_host_profile(vscode_ceiling_host())
            .build()
            .await
            .expect("ceiling v2 state");
    load_role_impl(&ceiling_v2_state, "mumu", true)
        .await
        .expect("load mumu");

    set_session_plugin_backend_impl(
        &ceiling_v2_state,
        &SetSessionPluginBackendRequest {
            role_id: "mumu".to_string(),
            module: "memory".to_string(),
            backend: Some(Some("remote".to_string())),
            local_memory_provider_id: None,
            session_id: Some("after-ceiling".to_string()),
        },
    )
    .await
    .expect("session override after ceiling");

    let overridden = get_plugin_resolution_debug_impl(
        &ceiling_v2_state,
        &GetPluginResolutionDebugRequest {
            role_id: "mumu".to_string(),
            session_id: Some("after-ceiling".to_string()),
        },
    )
    .await
    .expect("debug after override");
    assert_eq!(
        overridden.plugin_backends_effective.memory,
        MemoryBackend::Builtin,
        "host ceiling caps effective backends after session override merge"
    );
    assert_eq!(
        overridden.plugin_backends_effective_sources.memory,
        PluginBackendSource::SessionOverride,
        "source map still records session override intent"
    );
}

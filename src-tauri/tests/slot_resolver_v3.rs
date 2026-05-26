//! P3：`SlotResolver` 多实例解析与 `resolved_plugins_for_session` 走 effective registry。

#![allow(clippy::unwrap_used, clippy::expect_used)]

use oclive_kernel_runtime::domain::complex_emotion::ComplexEmotionInput;
use oclive_validation::{SlotOverridePatch, SlotRegistryEntry};
use oclivenewnew_tauri::domain::plugin_host::PluginHost;
use oclivenewnew_tauri::infrastructure::high_risk_grants::HighRiskGrantStore;
use oclivenewnew_tauri::infrastructure::llm::LlmClient;
use oclivenewnew_tauri::infrastructure::remote_fallback_policy::new_remote_fallback_switch;
use oclivenewnew_tauri::infrastructure::MockLlmClient;
use oclivenewnew_tauri::domain::slot_runner::SlotRunner;
use oclivenewnew_tauri::models::plugin_backends::LlmBackend;
use oclivenewnew_tauri::models::{MemoryBackend, Role};
use oclivenewnew_tauri::state::AppState;
use std::collections::BTreeMap;
use std::sync::Arc;

fn host() -> PluginHost {
    let llm: Arc<dyn LlmClient> = Arc::new(MockLlmClient {
        reply: String::new(),
    });
    let tmp = std::env::temp_dir();
    let grants = HighRiskGrantStore::load(tmp.clone(), false);
    let remote_fb = new_remote_fallback_switch(true);
    PluginHost::new(llm, None, tmp, grants, remote_fb)
}

#[test]
fn slot_resolver_lists_memory_instances_by_position() {
    let mut reg = BTreeMap::new();
    reg.insert(
        "mem_b".into(),
        SlotRegistryEntry {
            slot_type: "memory".into(),
            label: "b".into(),
            backend: "builtin".into(),
            position: 2,
            plugin: None,
            plugins: None,
            model: None,
            url: None,
            local_memory_provider_id: None,
            zone: None,
            policy: None,
        },
    );
    reg.insert(
        "mem_a".into(),
        SlotRegistryEntry {
            slot_type: "memory".into(),
            label: "a".into(),
            backend: "builtin_v2".into(),
            position: 1,
            plugin: None,
            plugins: None,
            model: None,
            url: None,
            local_memory_provider_id: None,
            zone: None,
            policy: None,
        },
    );
    let h = host();
    let pl = h.resolve_for_effective_backends(
        &oclive_validation::slot_registry_to_plugin_backends(&reg),
        Some(&reg),
        None,
    );
    let slots = pl.slots.expect("slots");
    assert_eq!(slots.memory.len(), 2);
    assert_eq!(slots.memory[0].0, "mem_a");
    assert_eq!(slots.memory[1].0, "mem_b");
    assert!(pl
        .complex_emotion
        .resolve_turn(&ComplexEmotionInput {
            role_id: "r".into(),
            scene_id: "s".into(),
            user_message: "hi".into(),
            bot_reply: String::new(),
            recent_dialogue_summary: None,
            previous_narrative_hint: String::new(),
            user_valence: None,
            user_dominance: None,
            previous_user_message: None,
        })
        .is_ok());
}

#[tokio::test]
async fn session_slot_override_changes_folded_memory_backend() {
    let llm: Arc<dyn LlmClient> = Arc::new(MockLlmClient {
        reply: String::new(),
    });
    let state = AppState::new_in_memory_with_llm(llm, std::env::temp_dir().join("roles-p3-test"))
        .await
        .expect("state");
    let mut reg = BTreeMap::new();
    reg.insert(
        "memory".into(),
        SlotRegistryEntry {
            slot_type: "memory".into(),
            label: "m".into(),
            backend: "builtin".into(),
            position: 1,
            plugin: None,
            plugins: None,
            model: None,
            url: None,
            local_memory_provider_id: None,
            zone: None,
            policy: None,
        },
    );
    let role = Role {
        id: "test-role".into(),
        plugin_backends: std::sync::Arc::new(oclive_validation::slot_registry_to_plugin_backends(&reg)),
        slot_registry: Some(reg),
        ..Default::default()
    };
    let ns = "srid-p3";
    state.set_session_slot_override(
        ns,
        "memory",
        SlotOverridePatch {
            backend: Some("builtin_v2".into()),
            ..Default::default()
        },
    );
    let eff = state.effective_plugin_backends_for_session(&role, ns);
    assert_eq!(eff.memory, MemoryBackend::BuiltinV2);
    let pl = state.resolved_plugins_for_session(&role, Some(ns));
    let expected = state.plugins.memory_retrieval_for_plugin_backends(&eff);
    assert!(Arc::ptr_eq(&pl.memory, &expected));
}

#[tokio::test]
async fn user_cloud_provider_overrides_blueprint_ollama_llm_slot() {
    let llm: Arc<dyn LlmClient> = Arc::new(MockLlmClient {
        reply: String::new(),
    });
    let state = AppState::new_in_memory_with_llm(llm, std::env::temp_dir().join("roles-p3-llm-cloud"))
        .await
        .expect("state");
    let mut reg = BTreeMap::new();
    reg.insert(
        "llm".into(),
        SlotRegistryEntry {
            slot_type: "llm".into(),
            label: "main".into(),
            backend: "ollama".into(),
            position: 1,
            plugin: None,
            plugins: None,
            model: Some("deepseek-chat".into()),
            url: None,
            local_memory_provider_id: None,
            zone: None,
            policy: None,
        },
    );
    let role = Role {
        id: "test-role".into(),
        plugin_backends: std::sync::Arc::new(oclive_validation::slot_registry_to_plugin_backends(
            &reg,
        )),
        slot_registry: Some(reg),
        ..Default::default()
    };
    let ns = "srid-cloud-llm";
    state
        .db_manager
        .upsert_app_setting("user_llm_provider", "cloud")
        .await
        .expect("provider");
    oclivenewnew_tauri::api::llm_settings::apply_user_llm_env(&state)
        .await
        .expect("apply env");
    let eff = state.effective_plugin_backends_for_session(&role, ns);
    assert_eq!(eff.llm, LlmBackend::Remote);
    let pl = state.resolved_plugins_for_session(&role, Some(ns));
    let primary = SlotRunner::primary_llm(&pl);
    let ollama = state.plugins.llm_for(LlmBackend::Ollama);
    assert!(
        !Arc::ptr_eq(&primary, &ollama),
        "blueprint ollama slot must not bind default Ollama when user provider is cloud"
    );
    assert_eq!(eff.llm, LlmBackend::Remote);
    assert!(pl.slots.is_some());
}

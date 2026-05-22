//! P4：双 memory / 双 llm 等多实例合并烟测。

#![allow(clippy::unwrap_used, clippy::expect_used)]

use oclive_validation::SlotRegistryEntry;
use oclivenewnew_tauri::domain::plugin_host::PluginHost;
use oclivenewnew_tauri::domain::slot_runner::SlotRunner;
use oclivenewnew_tauri::infrastructure::high_risk_grants::HighRiskGrantStore;
use oclivenewnew_tauri::infrastructure::llm::{LlmClient, MockLlmClient};
use oclivenewnew_tauri::infrastructure::remote_fallback_policy::new_remote_fallback_switch;
use oclivenewnew_tauri::models::{Memory, PluginBackends};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

struct CountingLlm {
    inner: MockLlmClient,
    calls: Arc<AtomicUsize>,
}

#[async_trait::async_trait]
impl LlmClient for CountingLlm {
    async fn generate(
        &self,
        model: &str,
        prompt: &str,
    ) -> oclivenewnew_tauri::error::Result<String> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.inner.generate(model, prompt).await
    }

    async fn generate_tag(
        &self,
        model: &str,
        prompt: &str,
    ) -> oclivenewnew_tauri::error::Result<String> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.inner.generate_tag(model, prompt).await
    }
}

fn host_with_llm(llm: Arc<dyn LlmClient>) -> PluginHost {
    let tmp = std::env::temp_dir();
    let grants = HighRiskGrantStore::load(tmp.clone(), false);
    let remote_fb = new_remote_fallback_switch(true);
    PluginHost::new(llm, None, tmp, grants, remote_fb)
}

#[test]
fn dual_memory_slots_merge_without_panic() {
    let mut reg = BTreeMap::new();
    reg.insert(
        "mem_a".into(),
        SlotRegistryEntry {
            slot_type: "memory".into(),
            label: "a".into(),
            backend: "builtin".into(),
            position: 1,
            plugin: None,
            plugins: None,
            model: None,
            url: None,
            local_memory_provider_id: None,
            zone: None,
        },
    );
    reg.insert(
        "mem_b".into(),
        SlotRegistryEntry {
            slot_type: "memory".into(),
            label: "b".into(),
            backend: "builtin_v2".into(),
            position: 2,
            plugin: None,
            plugins: None,
            model: None,
            url: None,
            local_memory_provider_id: None,
            zone: None,
        },
    );
    let pb = oclive_validation::slot_registry_to_plugin_backends(&reg);
    let h = host_with_llm(Arc::new(MockLlmClient {
        reply: String::new(),
    }));
    let pl = h.resolve_for_effective_backends(&pb, Some(&reg), None);
    let mems = vec![Memory {
        id: "x".into(),
        role_id: "r".into(),
        content: "c".into(),
        importance: 0.8,
        weight: 1.0,
        created_at: chrono::Utc::now(),
        scene_id: None,
    }];
    let ranked = SlotRunner::rank_memories(
        &pl,
        oclivenewnew_tauri::domain::memory_retrieval::MemoryRetrievalInput {
            memories: &mems,
            user_query: "q",
            scene_id: None,
            limit: 4,
        },
    )
    .expect("rank");
    assert!(!ranked.is_empty());
    assert!(pl.slots.is_some());
}

#[tokio::test]
async fn dual_llm_slots_call_both_serially() {
    let calls = Arc::new(AtomicUsize::new(0));
    let llm: Arc<dyn LlmClient> = Arc::new(CountingLlm {
        inner: MockLlmClient {
            reply: "from-mock".into(),
        },
        calls: calls.clone(),
    });
    let mut reg = BTreeMap::new();
    for (key, pos) in [("llm_a", 1), ("llm_b", 2)] {
        reg.insert(
            key.into(),
            SlotRegistryEntry {
                slot_type: "llm".into(),
                label: key.into(),
                backend: "ollama".into(),
                position: pos,
                plugin: None,
                plugins: None,
                model: None,
                url: None,
                local_memory_provider_id: None,
                zone: None,
            },
        );
    }
    let pb = PluginBackends::default();
    let h = host_with_llm(llm);
    let pl = h.resolve_for_effective_backends(&pb, Some(&reg), None);
    let reply = SlotRunner::generate_llm(&pl, "test-model", "hello")
        .await
        .expect("generate");
    assert_eq!(reply, "from-mock");
    assert_eq!(
        calls.load(Ordering::SeqCst),
        2,
        "both llm instances should run"
    );
}

//! 集成级烟测：`plugin_backends` 中 memory / emotion / event / prompt 为 `builtin_v2`、`llm` 为 `ollama` 时，
//! [`PluginHost::resolve_for_role`] 能解析 **六条子系统线**（含 `agent`，默认 `builtin`），不跑完整对话。
//!
//! `PluginHost::new` 使用临时应用数据根目录与宽松高风险授权夹具（与集成测 `AppState::new_in_memory*` 一致）。

#![allow(clippy::unwrap_used, clippy::expect_used)]

use oclivenewnew_tauri::domain::plugin_host::PluginHost;
use oclivenewnew_tauri::infrastructure::high_risk_grants::HighRiskGrantStore;
use oclivenewnew_tauri::infrastructure::llm::LlmClient;
use oclivenewnew_tauri::infrastructure::remote_fallback_policy::new_remote_fallback_switch;
use oclivenewnew_tauri::infrastructure::MockLlmClient;
use oclivenewnew_tauri::models::{
    EmotionBackend, EventBackend, LlmBackend, MemoryBackend, PluginBackends, PromptBackend, Role,
};
use std::sync::Arc;

#[test]
fn resolve_role_with_all_builtin_v2() {
    let llm: Arc<dyn LlmClient> = Arc::new(MockLlmClient {
        reply: String::new(),
    });
    let tmp = std::env::temp_dir();
    let grants = HighRiskGrantStore::load(tmp.clone(), false);
    let remote_fb = new_remote_fallback_switch(true);
    let host = PluginHost::new(llm, None, tmp, grants, remote_fb);
    let role = Role {
        plugin_backends: std::sync::Arc::new(PluginBackends {
            memory: MemoryBackend::BuiltinV2,
            emotion: EmotionBackend::BuiltinV2,
            event: EventBackend::BuiltinV2,
            prompt: PromptBackend::BuiltinV2,
            llm: LlmBackend::Ollama,
            ..Default::default()
        }),
        ..Default::default()
    };
    let _pl = host.resolve_for_role(&role);
}

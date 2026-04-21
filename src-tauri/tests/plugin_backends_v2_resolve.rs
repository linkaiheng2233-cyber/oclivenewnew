//! 集成级烟测：`plugin_backends` 全部为 `builtin_v2` 时 `PluginHost` 能解析五条线（不跑完整对话）。

use oclivenewnew_tauri::domain::plugin_host::PluginHost;
use oclivenewnew_tauri::infrastructure::llm::LlmClient;
use oclivenewnew_tauri::infrastructure::MockLlmClient;
use oclivenewnew_tauri::models::{
    EmotionBackend, EventBackend, LlmBackend, MemoryBackend, PluginBackends, PromptBackend, Role,
};
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn resolve_role_with_all_builtin_v2() {
    let llm: Arc<dyn LlmClient> = Arc::new(MockLlmClient {
        reply: String::new(),
    });
    let host = PluginHost::new(llm, None, PathBuf::from("."));
    let role = Role {
        plugin_backends: PluginBackends {
            memory: MemoryBackend::BuiltinV2,
            emotion: EmotionBackend::BuiltinV2,
            event: EventBackend::BuiltinV2,
            prompt: PromptBackend::BuiltinV2,
            llm: LlmBackend::Ollama,
            ..Default::default()
        },
        ..Default::default()
    };
    let _pl = host.resolve_for_role(&role);
}

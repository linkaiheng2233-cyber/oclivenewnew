//! 集成级烟测：`plugin_backends` 中 memory / emotion / event / prompt 为 `builtin_v2`、`llm` 为 `ollama` 时，
//! [`PluginHost::resolve_for_role`] 能解析 **六条子系统线**（含 `agent`，默认 `builtin`），不跑完整对话。
//!
//! `PluginHost::new` 第三参为应用数据根目录（生产环境为 Tauri app data；此处用 `std::env::temp_dir()`），
//! 供 MCP 配置扫描等基础设施使用。

use oclivenewnew_tauri::domain::plugin_host::PluginHost;
use oclivenewnew_tauri::infrastructure::llm::LlmClient;
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
    let host = PluginHost::new(llm, None, std::env::temp_dir());
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

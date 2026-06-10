//! 集成级烟测：`plugin_backends` 全槽 `builtin`、`llm` 为 `ollama` 时，
//! [`PluginHost::resolve_for_role`] 能解析 **六条子系统线**（含 `agent`，默认 `builtin`），不跑完整对话。
//!
//! `PluginHost::new` 使用临时应用数据根目录与宽松高风险授权夹具（与集成测 `AppState::new_in_memory*` 一致）。

#![allow(clippy::unwrap_used, clippy::expect_used)]

use oclive_kernel_host::infrastructure::plugin_wiring::test_plugin_host;
use oclive_kernel_types::models::{
    EmotionBackend, EventBackend, LlmBackend, MemoryBackend, PluginBackends, PromptBackend, Role,
};

#[test]
fn resolve_role_with_all_builtin() {
    let host = test_plugin_host();
    let role = Role {
        plugin_backends: std::sync::Arc::new(PluginBackends {
            memory: MemoryBackend::Builtin,
            emotion: EmotionBackend::Builtin,
            event: EventBackend::Builtin,
            prompt: PromptBackend::Builtin,
            llm: LlmBackend::Ollama,
            ..Default::default()
        }),
        ..Default::default()
    };
    let _pl = host.resolve_for_role(&role);
}

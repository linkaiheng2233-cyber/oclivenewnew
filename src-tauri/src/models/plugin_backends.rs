//! 角色包 `settings.json` → `plugin_backends`：编译期可选子系统实现（v1 枚举）。

use serde::{Deserialize, Serialize};

/// 记忆检索后端（见 `creator-docs/plugin-and-architecture/PLUGIN_V1.md`）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryBackend {
    #[default]
    Builtin,
    BuiltinV2,
    Remote,
}

/// 用户情绪分析后端
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmotionBackend {
    #[default]
    Builtin,
    /// 第二套内置：强中性偏置（用于验证 `emotion` 枚举可切换；不追求更高准确率）
    BuiltinV2,
    Remote,
}

/// 事件影响估计后端
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventBackend {
    #[default]
    Builtin,
    /// 第二套内置：在 builtin 估计结果上将 `impact_factor` 乘以 0.5（更保守）
    BuiltinV2,
    Remote,
}

/// Prompt 组装后端
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptBackend {
    #[default]
    Builtin,
    /// 第二套内置：在 builtin 正文前追加固定标记前缀（用于验证 `prompt` 枚举可切换）
    BuiltinV2,
    Remote,
}

/// 主对话 LLM 调用后端（[`LlmClient`](crate::infrastructure::llm::LlmClient)）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LlmBackend {
    /// 应用启动时注入的默认客户端（通常为 Ollama）
    #[default]
    Ollama,
    /// 侧车 HTTP JSON-RPC（`OCLIVE_REMOTE_LLM_URL`）；未配置时委托进程内默认 LLM 并记日志
    Remote,
}

/// 与 `DiskRoleSettings.plugin_backends` / 运行时 `Role.plugin_backends` 一致
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginBackends {
    #[serde(default)]
    pub memory: MemoryBackend,
    #[serde(default)]
    pub emotion: EmotionBackend,
    #[serde(default)]
    pub event: EventBackend,
    #[serde(default)]
    pub prompt: PromptBackend,
    #[serde(default)]
    pub llm: LlmBackend,
}

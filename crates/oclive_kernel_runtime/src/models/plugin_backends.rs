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
    /// 本地已注册的 memory provider（见 `_local_plugins`）；执行未接入时委托 `builtin_v2` 排序逻辑。
    Local,
    /// `plugins/<id>/` 目录插件子进程 JSON-RPC（`plugin_backends.directory_plugins.memory` 指定 id）。
    Directory,
    /// 静音 Provider：检索无结果、不注入上下文（见 `MODULE_NONE_SEMANTICS.md` §1）。
    None,
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
    Directory,
    /// 静音 Provider：强中性分布（见 `MODULE_NONE_SEMANTICS.md` §2）。
    None,
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
    Directory,
    /// 静音 Provider：`Ignore` / 零影响（见 `MODULE_NONE_SEMANTICS.md` §3）。
    None,
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
    Directory,
    /// 静音 Provider：最小占位 prompt（见 `MODULE_NONE_SEMANTICS.md` §4）。
    None,
}

/// 复杂情感复盘后端（`complex_emotion.resolve_turn`）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplexEmotionBackend {
    #[default]
    Builtin,
    /// 独立 Remote 侧车（`OCLIVE_COMPLEX_EMOTION_URL`；兼容旧名 `OCLIVE_REMOTE_COMPLEX_EMOTION_URL`）
    Remote,
    /// `plugins/<id>/` 目录插件子进程 JSON-RPC（`plugin_backends.directory_plugins.complex_emotion` 指定 id）。
    Directory,
    /// 空 Provider：不做复盘，不向 prompt 注入。
    None,
}

/// Agent 任务编排后端（工具调度 / ReAct）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentBackend {
    #[default]
    Builtin,
    Remote,
    Directory,
    /// 静音 Provider：不执行 ReAct / 工具链；`AgentProvider::process` 返回固定提示（见 `MODULE_NONE_SEMANTICS.md` §7）。
    None,
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
    /// 目录插件子进程 JSON-RPC（`plugin_backends.directory_plugins.llm` 指定 manifest `id`）。
    Directory,
    /// 静音 Provider：拒绝生成（见 `MODULE_NONE_SEMANTICS.md` §5）。
    None,
}

/// 各模块使用 `*_backend = directory` 时对应的插件 manifest `id`。
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DirectoryPluginSlots {
    #[serde(default)]
    pub memory: Option<String>,
    #[serde(default)]
    pub emotion: Option<String>,
    #[serde(default)]
    pub event: Option<String>,
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub llm: Option<String>,
    #[serde(default)]
    pub agent: Option<String>,
    #[serde(default)]
    pub complex_emotion: Option<String>,
}

impl DirectoryPluginSlots {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.memory.as_ref().is_none_or(|s| s.trim().is_empty())
            && self.emotion.as_ref().is_none_or(|s| s.trim().is_empty())
            && self.event.as_ref().is_none_or(|s| s.trim().is_empty())
            && self.prompt.as_ref().is_none_or(|s| s.trim().is_empty())
            && self.llm.as_ref().is_none_or(|s| s.trim().is_empty())
            && self.agent.as_ref().is_none_or(|s| s.trim().is_empty())
            && self
                .complex_emotion
                .as_ref()
                .is_none_or(|s| s.trim().is_empty())
    }
}

/// 与 `DiskRoleSettings.plugin_backends` / 运行时 `Role.plugin_backends` 一致
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginBackends {
    #[serde(default)]
    pub memory: MemoryBackend,
    /// `memory = local` 时可选：指定已注册的 `provider_id`（trim 后非空才参与匹配）。多 provider 且省略时按字典序取第一个并记歧义日志。
    #[serde(default)]
    pub local_memory_provider_id: Option<String>,
    #[serde(default)]
    pub emotion: EmotionBackend,
    #[serde(default)]
    pub event: EventBackend,
    #[serde(default)]
    pub prompt: PromptBackend,
    #[serde(default)]
    pub llm: LlmBackend,
    #[serde(default)]
    pub agent: AgentBackend,
    #[serde(default)]
    pub complex_emotion: ComplexEmotionBackend,
    /// `memory` / `emotion` / `event` / `prompt` / `llm` 为 `directory` 时在此给出插件 id。
    #[serde(default)]
    pub directory_plugins: DirectoryPluginSlots,
}

/// 有效后端来源：角色包默认、会话覆盖、或环境变量覆盖（当前仅 LLM 可能为 env）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginBackendSource {
    #[default]
    PackDefault,
    SessionOverride,
    EnvOverride,
}

/// 各模块当前有效后端来源快照（与 `PluginBackends` 字段一一对应）。
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginBackendsSourceMap {
    #[serde(default)]
    pub memory: PluginBackendSource,
    #[serde(default)]
    pub emotion: PluginBackendSource,
    #[serde(default)]
    pub event: PluginBackendSource,
    #[serde(default)]
    pub prompt: PluginBackendSource,
    #[serde(default)]
    pub llm: PluginBackendSource,
    #[serde(default)]
    pub agent: PluginBackendSource,
    #[serde(default)]
    pub complex_emotion: PluginBackendSource,
}

/// 会话级覆盖：仅 `Some` 字段会替换角色包 `plugin_backends` 对应模块。
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginBackendsOverride {
    #[serde(default)]
    pub memory: Option<MemoryBackend>,
    /// 会话覆盖：存在该字段时替换包内 `local_memory_provider_id`（`Some("")` 经 trim 后为空则清空为 `None`）。
    #[serde(default)]
    pub local_memory_provider_id: Option<String>,
    #[serde(default)]
    pub emotion: Option<EmotionBackend>,
    #[serde(default)]
    pub event: Option<EventBackend>,
    #[serde(default)]
    pub prompt: Option<PromptBackend>,
    #[serde(default)]
    pub llm: Option<LlmBackend>,
    #[serde(default)]
    pub agent: Option<AgentBackend>,
    #[serde(default)]
    pub complex_emotion: Option<ComplexEmotionBackend>,
    /// 会话级覆盖目录插件槽位（`Some` 时与包内字段按槽合并，见 [`Self::apply_to`]）。
    #[serde(default)]
    pub directory_plugins: Option<DirectoryPluginSlots>,
    /// 合并目录槽后，强制清空 `directory_plugins.llm`（专家图切到云端 `Remote` 时需要卸下本地 llama 槽位）。
    #[serde(default)]
    pub force_clear_directory_llm_slot: bool,
}

impl PluginBackendsOverride {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.memory.is_none()
            && self.local_memory_provider_id.is_none()
            && self.emotion.is_none()
            && self.event.is_none()
            && self.prompt.is_none()
            && self.llm.is_none()
            && self.agent.is_none()
            && self.complex_emotion.is_none()
            && self.directory_plugins.is_none()
            && !self.force_clear_directory_llm_slot
    }

    #[must_use]
    pub fn apply_to(&self, base: &PluginBackends) -> PluginBackends {
        let local_memory_provider_id = match &self.local_memory_provider_id {
            None => base.local_memory_provider_id.clone(),
            Some(s) => {
                let t = s.trim();
                if t.is_empty() {
                    None
                } else {
                    Some(t.to_string())
                }
            }
        };
        let mut directory_plugins = match &self.directory_plugins {
            None => base.directory_plugins.clone(),
            Some(ov) => DirectoryPluginSlots {
                memory: trimmed_or_fallback(
                    ov.memory.as_deref(),
                    base.directory_plugins.memory.as_deref(),
                ),
                emotion: trimmed_or_fallback(
                    ov.emotion.as_deref(),
                    base.directory_plugins.emotion.as_deref(),
                ),
                event: trimmed_or_fallback(
                    ov.event.as_deref(),
                    base.directory_plugins.event.as_deref(),
                ),
                prompt: trimmed_or_fallback(
                    ov.prompt.as_deref(),
                    base.directory_plugins.prompt.as_deref(),
                ),
                llm: trimmed_or_fallback(ov.llm.as_deref(), base.directory_plugins.llm.as_deref()),
                agent: trimmed_or_fallback(
                    ov.agent.as_deref(),
                    base.directory_plugins.agent.as_deref(),
                ),
                complex_emotion: trimmed_or_fallback(
                    ov.complex_emotion.as_deref(),
                    base.directory_plugins.complex_emotion.as_deref(),
                ),
            },
        };
        if self.force_clear_directory_llm_slot {
            directory_plugins.llm = None;
        }
        PluginBackends {
            memory: self.memory.unwrap_or(base.memory),
            local_memory_provider_id,
            emotion: self.emotion.unwrap_or(base.emotion),
            event: self.event.unwrap_or(base.event),
            prompt: self.prompt.unwrap_or(base.prompt),
            llm: self.llm.unwrap_or(base.llm),
            agent: self.agent.unwrap_or(base.agent),
            complex_emotion: self.complex_emotion.unwrap_or(base.complex_emotion),
            directory_plugins,
        }
    }
}

fn trimmed_or_fallback(ov: Option<&str>, base: Option<&str>) -> Option<String> {
    match ov {
        None => base.map(|s| s.to_string()),
        Some(s) => {
            let t = s.trim();
            if t.is_empty() {
                base.map(|x| x.to_string())
            } else {
                Some(t.to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn override_replaces_local_memory_provider_id() {
        let base = PluginBackends {
            memory: MemoryBackend::Local,
            local_memory_provider_id: Some("a".into()),
            ..Default::default()
        };
        let ov = PluginBackendsOverride {
            local_memory_provider_id: Some("b".into()),
            ..Default::default()
        };
        let eff = ov.apply_to(&base);
        assert_eq!(eff.local_memory_provider_id.as_deref(), Some("b"));
        assert_eq!(eff.memory, MemoryBackend::Local);
    }

    #[test]
    fn override_whitespace_local_memory_provider_id_clears() {
        let base = PluginBackends {
            memory: MemoryBackend::Local,
            local_memory_provider_id: Some("a".into()),
            ..Default::default()
        };
        let ov = PluginBackendsOverride {
            local_memory_provider_id: Some("   ".into()),
            ..Default::default()
        };
        let eff = ov.apply_to(&base);
        assert!(eff.local_memory_provider_id.is_none());
    }
}

//! Module 9: Expert Models (ExpertGraph) + PromptStyle override.
//!
//! M1 stores a minimal closed-loop: pick base GGUF + multiple LoRAs with strengths + optional PromptStyle override.
//! M2 evolves the structure into a node-graph, keeping compilation as a separate pipeline.

use serde::{Deserialize, Serialize};

/// Source indicator for effective config resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExpertConfigSource {
    PackDefault,
    RoleDefault,
    SessionOverride,
}

/// A minimal prompt style override layer.
///
/// Semantics: `None` means "no override" for that field.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptStyleOverride {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_quality_anchor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub core_personality: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Node-graph foundation (M2): nodes + edges.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpertGraph {
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub nodes: Vec<ExpertNode>,
    #[serde(default)]
    pub edges: Vec<ExpertEdge>,
}

impl Default for ExpertGraph {
    fn default() -> Self {
        Self {
            version: 1,
            nodes: vec![],
            edges: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpertEdge {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpertNodeUi {
    #[serde(default)]
    pub x: f32,
    #[serde(default)]
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExpertNode {
    BaseModel {
        id: String,
        gguf_path: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        ui: Option<ExpertNodeUi>,
    },
    LoraAdapter {
        id: String,
        gguf_path: String,
        #[serde(default = "default_lora_strength")]
        strength: f32,
        #[serde(default = "default_true")]
        enabled: bool,
        #[serde(default)]
        order: i32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        ui: Option<ExpertNodeUi>,
    },
    PromptStyle {
        id: String,
        style: PromptStyleOverride,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        ui: Option<ExpertNodeUi>,
    },
    /// 引用宿主已配置的 OpenAI 兼容云端 LLM（`host_cloud_llm_json` / 环境变量）；可选 `model` 覆盖请求体中的模型 id。
    CloudModel {
        id: String,
        /// 仅支持 `"host"`：与宿主全局 cloud 配置一致。
        #[serde(default = "default_cloud_host_source")]
        host_source: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        model: Option<String>,
        #[serde(default = "default_true")]
        enabled: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        ui: Option<ExpertNodeUi>,
    },
    /// 回合结束后：若用户句或模型回复包含 `match_substring`，写入一条长期记忆。
    EventTrigger {
        id: String,
        match_substring: String,
        memory_content: String,
        #[serde(default = "default_event_importance")]
        importance: f32,
        #[serde(default = "default_true")]
        enabled: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        ui: Option<ExpertNodeUi>,
    },
}

fn default_cloud_host_source() -> String {
    "host".to_string()
}

fn default_event_importance() -> f32 {
    0.75
}

fn default_lora_strength() -> f32 {
    1.0
}

fn default_true() -> bool {
    true
}

/// A compiled view for llama sidecar config.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlamaLocalPluginConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_path: Option<String>,
    /// Whitespace-separated args forwarded to llama-server by the sidecar.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub llama_args: Option<String>,
}

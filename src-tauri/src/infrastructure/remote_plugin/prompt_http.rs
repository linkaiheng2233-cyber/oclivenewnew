//! JSON-RPC：`prompt.build_prompt` — 参数为可序列化的上下文快照（含完整 `Role`）。

use crate::domain::prompt_assembler::PromptAssembler;
use crate::domain::prompt_builder::PromptInput;
use crate::domain::BuiltinPromptAssembler;
use crate::infrastructure::remote_plugin::config::RemotePluginHttpConfig;
use crate::infrastructure::remote_plugin::jsonrpc::{self, RemoteRpcChannel};
use crate::models::{PersonalitySource, Role};
use serde_json::json;

const METHOD_PROMPT_BUILD: &str = "prompt.build_prompt";
const METHOD_PROMPT_TOPIC_HINT: &str = "prompt.top_topic_hint";

pub struct RemotePromptAssemblerHttp {
    client: reqwest::blocking::Client,
    cfg: RemotePluginHttpConfig,
    fallback: BuiltinPromptAssembler,
}

impl RemotePromptAssemblerHttp {
    pub fn new(cfg: RemotePluginHttpConfig) -> Self {
        let client = reqwest::blocking::Client::builder()
            .connect_timeout(cfg.connect_timeout())
            .timeout(cfg.timeout)
            .build()
            .expect("reqwest blocking client");
        Self {
            client,
            cfg,
            fallback: BuiltinPromptAssembler,
        }
    }
}

impl PromptAssembler for RemotePromptAssemblerHttp {
    fn build_prompt(&self, input: &PromptInput<'_>) -> String {
        let params = match serde_json::to_value(PromptInputSnapshot::from_input(input)) {
            Ok(v) => v,
            Err(e) => {
                log::warn!(
                    target: "oclive_plugin",
                    "prompt snapshot serialize failed: {}; builtin",
                    e
                );
                return self.fallback.build_prompt(input);
            }
        };
        match jsonrpc::call_blocking(
            RemoteRpcChannel::Plugin,
            &self.client,
            &self.cfg.endpoint,
            METHOD_PROMPT_BUILD,
            params,
            self.cfg.bearer_token.as_deref(),
        ) {
            Ok(v) => {
                if let Some(s) = v.get("prompt").and_then(|x| x.as_str()) {
                    return s.to_string();
                }
                if let Some(s) = v.as_str() {
                    return s.to_string();
                }
                log::warn!(target: "oclive_plugin", "prompt.build_prompt: bad shape; builtin");
                self.fallback.build_prompt(input)
            }
            Err(e) => {
                log::warn!(
                    target: "oclive_plugin",
                    "prompt.build_prompt remote failed endpoint={} err={}; fallback=builtin",
                    self.cfg.endpoint,
                    e
                );
                self.fallback.build_prompt(input)
            }
        }
    }

    fn top_topic_hint(&self, role: &Role, scene_id: &str) -> Option<String> {
        let params = json!({
            "role": role,
            "scene_id": scene_id,
        });
        match jsonrpc::call_blocking(
            RemoteRpcChannel::Plugin,
            &self.client,
            &self.cfg.endpoint,
            METHOD_PROMPT_TOPIC_HINT,
            params,
            self.cfg.bearer_token.as_deref(),
        ) {
            Ok(v) => v
                .get("hint")
                .and_then(|x| x.as_str())
                .map(String::from)
                .or_else(|| v.as_str().map(String::from)),
            Err(_) => self.fallback.top_topic_hint(role, scene_id),
        }
    }
}

#[derive(serde::Serialize)]
struct PromptInputSnapshot<'a> {
    #[serde(flatten)]
    flat: PromptInputFlat<'a>,
}

#[derive(serde::Serialize)]
struct PromptInputFlat<'a> {
    role: &'a Role,
    /// 与 `role.evolution_config.personality_source` 一致；便于侧车不必从嵌套 `role` 解析。
    personality_source: PersonalitySource,
    personality: &'a crate::models::PersonalityVector,
    memories: &'a [crate::models::Memory],
    user_input: &'a str,
    user_emotion: &'a str,
    user_relation_id: &'a str,
    relation_hint: &'a str,
    relation_before: &'a str,
    favorability_before: f64,
    relation_preview: &'a str,
    favorability_preview: f64,
    event_type: &'a crate::models::EventType,
    impact_factor: f64,
    scene_label: &'a str,
    scene_detail: &'a str,
    topic_hint_line: &'a str,
    life_context_line: &'a str,
    worldview_snippet: &'a str,
    mutable_personality: &'a str,
    reply_quality_anchor: &'a str,
}

impl<'a> PromptInputSnapshot<'a> {
    fn from_input(input: &'a PromptInput<'_>) -> Self {
        Self {
            flat: PromptInputFlat {
                role: input.role,
                personality_source: input.role.evolution_config.personality_source,
                personality: input.personality,
                memories: input.memories,
                user_input: input.user_input,
                user_emotion: input.user_emotion,
                user_relation_id: input.user_relation_id,
                relation_hint: input.relation_hint,
                relation_before: input.relation_before,
                favorability_before: input.favorability_before,
                relation_preview: input.relation_preview,
                favorability_preview: input.favorability_preview,
                event_type: input.event_type,
                impact_factor: input.impact_factor,
                scene_label: input.scene_label,
                scene_detail: input.scene_detail,
                topic_hint_line: input.topic_hint_line,
                life_context_line: input.life_context_line,
                worldview_snippet: input.worldview_snippet,
                mutable_personality: input.mutable_personality,
                reply_quality_anchor: input.reply_quality_anchor,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn method_names_match_remote_protocol() {
        assert_eq!(METHOD_PROMPT_BUILD, "prompt.build_prompt");
        assert_eq!(METHOD_PROMPT_TOPIC_HINT, "prompt.top_topic_hint");
    }
}

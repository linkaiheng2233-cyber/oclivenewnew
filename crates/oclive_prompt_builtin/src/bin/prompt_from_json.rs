//! 从 stdin 读取一帧 JSON（与宿主 `prompt.build_prompt` 的 `params` 形状兼容），向 stdout 输出 `{"prompt":"..."}`。
//!
//! 供 `examples/oclive-prompt-builtin-directory` 通过子进程调用，与进程内 `PromptBuilder` 一致。

use oclive_kernel_core::models::Memory;
use oclive_kernel_core::prompt::{PromptInput, PromptRolePromptSlice};
use oclive_kernel_models::{
    EventType, EvolutionConfig, MemoryConfig, PersonalityVector, UserRelation,
};
use oclive_prompt_builtin::PromptBuilder;
use serde::Deserialize;
use std::io::{Read, Write};

#[derive(Debug, Deserialize)]
struct RoleSubset {
    name: String,
    description: String,
    core_personality: String,
    evolution_config: EvolutionConfig,
    #[serde(default)]
    user_relations: Vec<UserRelation>,
    #[serde(default)]
    memory_config: Option<MemoryConfig>,
    #[serde(default)]
    reply_quality_anchor: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PromptBuildRpcParams {
    role: RoleSubset,
    personality: PersonalityVector,
    #[serde(default)]
    memories: Vec<Memory>,
    #[serde(default)]
    user_input: String,
    #[serde(default)]
    user_emotion: String,
    #[serde(default)]
    user_relation_id: String,
    #[serde(default)]
    relation_hint: String,
    #[serde(default)]
    relation_before: String,
    #[serde(default)]
    favorability_before: f64,
    #[serde(default)]
    relation_preview: String,
    #[serde(default)]
    favorability_preview: f64,
    event_type: EventType,
    #[serde(default)]
    impact_factor: f64,
    #[serde(default)]
    scene_label: String,
    #[serde(default)]
    scene_detail: String,
    #[serde(default)]
    topic_hint_line: String,
    #[serde(default)]
    life_context_line: String,
    #[serde(default)]
    worldview_snippet: String,
    #[serde(default)]
    mutable_personality: String,
    #[serde(default)]
    reply_quality_anchor: String,
    #[serde(default)]
    complex_emotion_hint: Option<String>,
}

fn build(params: &PromptBuildRpcParams) -> String {
    let r = &params.role;
    let slice = PromptRolePromptSlice {
        name: r.name.as_str(),
        description: r.description.as_str(),
        core_personality: r.core_personality.as_str(),
        evolution_config: &r.evolution_config,
        user_relations: r.user_relations.as_slice(),
        memory_config: r.memory_config.as_ref(),
        reply_quality_anchor: r.reply_quality_anchor.as_deref(),
    };
    let input = PromptInput {
        role_any: &(),
        role_prompt: slice,
        personality: &params.personality,
        memories: &params.memories,
        user_input: &params.user_input,
        user_emotion: &params.user_emotion,
        user_relation_id: &params.user_relation_id,
        relation_hint: &params.relation_hint,
        relation_before: &params.relation_before,
        favorability_before: params.favorability_before,
        relation_preview: &params.relation_preview,
        favorability_preview: params.favorability_preview,
        event_type: &params.event_type,
        impact_factor: params.impact_factor,
        scene_label: &params.scene_label,
        scene_detail: &params.scene_detail,
        topic_hint_line: &params.topic_hint_line,
        life_context_line: &params.life_context_line,
        worldview_snippet: &params.worldview_snippet,
        mutable_personality: &params.mutable_personality,
        reply_quality_anchor: &params.reply_quality_anchor,
        complex_emotion_hint: params.complex_emotion_hint.as_deref(),
    };
    PromptBuilder::build_prompt(&input)
}

fn main() {
    let mut buf = String::new();
    if let Err(e) = std::io::stdin().read_to_string(&mut buf) {
        let _ = writeln!(std::io::stderr(), "oclive_prompt_from_json: read stdin: {e}");
        std::process::exit(1);
    }
    let params: PromptBuildRpcParams = match serde_json::from_str(buf.trim()) {
        Ok(p) => p,
        Err(e) => {
            let _ = writeln!(std::io::stderr(), "oclive_prompt_from_json: parse json: {e}");
            std::process::exit(1);
        }
    };
    let prompt = build(&params);
    println!("{}", serde_json::json!({ "prompt": prompt }));
}

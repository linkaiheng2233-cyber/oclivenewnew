//! `classic` 关闭时的轻量桩：与设施 crate 其他 `stub` 一致，供极简宿主链接。

use oclive_kernel_core::prompt::{PromptInput, TopicHintContext};

/// 轻量桩：[`build_prompt`](Self::build_prompt) 返回空串；[`top_topic_hint`](Self::top_topic_hint) 返回 `None`。
pub struct PromptBuilder;

impl PromptBuilder {
    pub fn build_prompt(_input: &PromptInput<'_>) -> String {
        String::new()
    }

    pub fn build_simple_prompt(role_name: &str, user_input: &str) -> String {
        format!("你是{role_name}。用户说: {user_input}\n请自然地回复。")
    }

    pub fn build_system_prompt(role_name: &str) -> String {
        format!(
            "你是一个名叫{role_name}的AI角色。请以这个角色的身份进行对话，保持一致的性格和语气。"
        )
    }

    pub fn build_guidance_prompt(core_personality: &str) -> String {
        format!("你的核心性格是: {core_personality}\n请根据这个性格特征来指导你的回复。")
    }

    pub fn top_topic_hint(_ctx: &TopicHintContext<'_>, _scene_id: &str) -> Option<String> {
        None
    }
}

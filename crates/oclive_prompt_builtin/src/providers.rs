//! `feature = "providers"`：进程内 Builtin / BuiltinV2 [`PromptAssembler`](oclive_kernel_core::prompt::PromptAssembler)。

use crate::PromptBuilder;
use oclive_kernel_core::prompt::{PromptAssembler, PromptInput, TopicHintContext};

/// 第二套内置与 V1 正文一致，但在正文前追加本前缀（可测差异）。
pub const PROMPT_BACKEND_V2_PREFIX: &str = "[oclive:prompt:builtin_v2]\n";

/// 内置：与历史 `PromptBuilder` 行为一致。
pub struct BuiltinPromptAssembler;

impl PromptAssembler for BuiltinPromptAssembler {
    fn build_prompt(&self, input: &PromptInput<'_>) -> String {
        PromptBuilder::build_prompt(input)
    }

    fn top_topic_hint(&self, ctx: &TopicHintContext<'_>, scene_id: &str) -> Option<String> {
        PromptBuilder::top_topic_hint(ctx, scene_id)
    }
}

/// 第二套内置：与 [`BuiltinPromptAssembler`] 相同逻辑，但在正文前追加固定前缀。
pub struct BuiltinPromptAssemblerV2;

impl PromptAssembler for BuiltinPromptAssemblerV2 {
    fn build_prompt(&self, input: &PromptInput<'_>) -> String {
        format!(
            "{}{}",
            PROMPT_BACKEND_V2_PREFIX,
            PromptBuilder::build_prompt(input)
        )
    }

    fn top_topic_hint(&self, ctx: &TopicHintContext<'_>, scene_id: &str) -> Option<String> {
        PromptBuilder::top_topic_hint(ctx, scene_id)
    }
}

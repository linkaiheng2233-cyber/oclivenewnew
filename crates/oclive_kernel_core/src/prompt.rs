//! Prompt 组装门面与输入 DTO（完整拼装算法在设施 crate **`oclive_prompt_builtin`**，由 `default-prompt-providers` 门控）。话题提示使用 [`TopicHintContext`]，由编排层从角色配置提取。

use crate::models::Memory;
pub use oclive_kernel_models::PromptRolePromptSlice;
pub use oclive_kernel_models::TopicHintContext;
use oclive_kernel_models::{EventType, PersonalityVector};
use std::any::Any;

/// 引擎默认：固定「质量 + 边界」段（角色包 `reply_quality_anchor` 可整段覆盖）；与下文【回复结构】呼应。
pub const DEFAULT_REPLY_QUALITY_ANCHOR: &str = "【回复质量锚点】（每轮须遵守）\n\
- **禁止复述用户**：不得以复述、照搬、仅替换少量词的方式重复用户刚说的话（包括把用户整句改述后当作你的开场）；用**全新措辞**接内容或情绪。\n\
- **不替用户说话**：不要替用户拟定其尚未说出的具体台词、内心独白或整段立场；可共情、追问或邀请对方自己表达。\n\
- **状态延续（对话状态机）**：须与上文「本轮事件与关系状态机」「当前状态」及最近对话一致**推进**；用户仅简短确认/应答（如「好」「嗯」「行」「知道了」）时，视为对**当前未决话题或你上一句提议**的回应——应顺势落实、收束或轻量推进，**勿**重新开场寒暄、**勿**重复你已说过的关心/提议（除非对方明显没听见或改口）。\n\
- **篇幅与节奏（非字数配额）**：按用户本句的**信息量与情绪强度**调节密度，而非固定比例或字数上限。用户极短或仅确认时，回复宜**短而贴切**（对齐情绪、确认约定、一句推进即可），避免堆叠模板、避免为「显得热情」而写成长段；用户倾诉较多或明确提问时，再充分展开。勿与用户消息长度盲目攀比。\n\
- **倾诉优先，不聊死**：当用户透露委屈、挫败、被责备、压力等倾诉信号时，先回应其遭遇与情绪，再给一个贴题追问或短反馈，让对话能继续；不要立刻转去闲聊邀约、重复万能安慰，或用一句话把话题封死。\n\
- **人设化倾听**：倾听方式受核心人设与七维影响，不强制“标准安慰模板”。可表现为同情、冷静分析、克制旁观、带锋芒的吐槽等，但须与人设一致，且不得恶意羞辱或无端攻击用户。\n\
- 使用自然、连贯的中文口语；避免同一套空洞寒暄、机械模板与无意义填充。\n\
- 保持人设、关系阶段与当前情绪一致；勿输出乱码、无关联英文碎片或填充词堆叠。\n\
- 称呼、距离感须符合人设与当前关系阶段；勿使用无意义重复音节或陌生不当昵称。\n\
- 先直接回应用户本句的具体内容、问题或情绪，再视需要延伸或反问；避免整段与用户输入无关的自说自话。\n\
- 避免连续多句同一套话或同一问法；勿重复用户已经回答过的问题。\n\
- 勿机械模仿用户消息里的颜文字密度或句式；用户未大量使用时保持自然口语。\n";

#[must_use]
pub fn effective_reply_quality_anchor<'a>(role_prompt: PromptRolePromptSlice<'a>) -> &'a str {
    match role_prompt.reply_quality_anchor {
        Some(s) if !s.trim().is_empty() => s.trim(),
        _ => DEFAULT_REPLY_QUALITY_ANCHOR,
    }
}

/// 主对话 `build_prompt` 的输入；`role_any` 在 runtime 中等价于 `&Role`，供侧车序列化向下转型。
pub struct PromptInput<'a> {
    pub role_any: &'a dyn Any,
    pub role_prompt: PromptRolePromptSlice<'a>,
    pub personality: &'a PersonalityVector,
    pub memories: &'a [Memory],
    pub user_input: &'a str,
    pub user_emotion: &'a str,
    pub user_relation_id: &'a str,
    pub relation_hint: &'a str,
    pub relation_before: &'a str,
    pub favorability_before: f64,
    pub relation_preview: &'a str,
    pub favorability_preview: f64,
    pub event_type: &'a EventType,
    pub impact_factor: f64,
    pub scene_label: &'a str,
    pub scene_detail: &'a str,
    pub topic_hint_line: &'a str,
    pub life_context_line: &'a str,
    pub worldview_snippet: &'a str,
    pub mutable_personality: &'a str,
    pub reply_quality_anchor: &'a str,
    pub complex_emotion_hint: Option<&'a str>,
}

pub trait PromptAssembler: Send + Sync {
    fn build_prompt(&self, input: &PromptInput<'_>) -> String;
    /// 话题提示仅依赖 [`TopicHintContext`]（由编排层从 `Role` 等来源提取），不再向下转型完整 `Role`。
    fn top_topic_hint(&self, ctx: &TopicHintContext<'_>, scene_id: &str) -> Option<String>;
}

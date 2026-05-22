//! Prompt 组装输入（纯数据结构）。

use crate::models::{EventType, Memory, PersonalityVector, Role};

/// 主对话 `build_prompt` 的输入，避免长参数列表与调用处错位。
pub struct PromptInput<'a> {
    pub role: &'a Role,
    pub personality: &'a PersonalityVector,
    pub memories: &'a [Memory],
    pub user_input: &'a str,
    pub user_emotion: &'a str,
    /// 当前用户身份键（与 manifest `user_relations`、DB 一致）；空则跳过【用户身份】整段。
    pub user_relation_id: &'a str,
    pub relation_hint: &'a str,
    pub relation_before: &'a str,
    pub favorability_before: f64,
    pub relation_preview: &'a str,
    pub favorability_preview: f64,
    pub event_type: &'a EventType,
    pub impact_factor: f64,
    pub scene_label: &'a str,
    /// 来自角色包 `description.txt` 或 `scene.json` 的自动拼装，新场景无需改代码。
    pub scene_detail: &'a str,
    pub topic_hint_line: &'a str,
    /// 虚拟时间日程推断一行；空则跳过（不改变无配置时的对话行为）
    pub life_context_line: &'a str,
    /// 本回合检索到的世界观知识片段；空则跳过【世界观设定】段
    pub worldview_snippet: &'a str,
    /// 人设优先模式下 DB 中的「可变性格档案」全文；`vector` 模式传空串即可。
    pub mutable_personality: &'a str,
    /// 合并后的「回复质量锚点」（引擎默认或 `settings.json` 覆盖）；注入在「用户说」之前。
    pub reply_quality_anchor: &'a str,
    /// 上一回合内置复杂情感模块输出的 `narrative_hint`；空则跳过【复杂情感叙事提示】段。
    pub previous_complex_emotion_narrative_hint: &'a str,
}

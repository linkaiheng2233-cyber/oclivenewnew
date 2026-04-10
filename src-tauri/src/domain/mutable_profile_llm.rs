//! 用 LLM 维护「可变性格档案」（`mutable_personality`）。
//!
//! **设计边界**（产品共识）  
//! - **核心性格档案**：`Role::core_personality`，仅由用户与创作者决定；**任何模型都不得改写**，以免幻觉或漂移把角色带偏。  
//! - **可变性格档案**：本模块负责的 DB 正文；模型在有限自主权内根据对话**抓住自己的相处侧变化**，幅度受 `EvolutionConfig` 约束。  
//! - 创作者不能手写运行中的可变档案进程，只能通过配置调节影响强弱。

use crate::error::Result;
use crate::infrastructure::llm::LlmClient;
use crate::models::{EventType, EvolutionConfig};
use std::sync::Arc;

use super::profile_personality::trim_mutable_storage;

pub struct MutableEvolutionInput<'a> {
    pub role_name: &'a str,
    pub core_personality: &'a str,
    pub prev_mutable: &'a str,
    pub user_message: &'a str,
    pub bot_reply: &'a str,
    pub user_emotion: &'a str,
    pub event_type: &'a EventType,
    /// 与七维模式一致：`estimate.impact_factor * role_runtime.event_impact_factor`，约 -1～1
    pub impact_scaled: f64,
    pub evolution: &'a EvolutionConfig,
}

fn max_change_instruction(m: f64) -> &'static str {
    if m <= 0.02 {
        "极保守：除非对话里信号非常明显，否则几乎不改档案措辞。"
    } else if m <= 0.05 {
        "保守：只做细小补充或微调，避免戏剧化转折。"
    } else if m <= 0.12 {
        "中等：可合并、删改与新事实明显矛盾的旧句，允许温和可见的态度变化。"
    } else {
        "在仍锚定核心的前提下，允许较明显的信任/态度变化；禁止突然 OOC 或编造包外硬设定。"
    }
}

fn truncate_chars(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max).collect();
    out.push('…');
    out
}

fn build_prompt(input: &MutableEvolutionInput<'_>) -> String {
    let core = truncate_chars(input.core_personality.trim(), 3500);
    let prev_raw = input.prev_mutable.trim();
    let prev = truncate_chars(prev_raw, 6000);
    let um = truncate_chars(input.user_message.trim(), 2000);
    let br = truncate_chars(input.bot_reply.trim(), 2000);
    let step = max_change_instruction(input.evolution.max_change_per_event);
    let prev_block = if prev.trim().is_empty() {
        "（尚无档案。若本回合有相处信号，写极简种子：1～3 短句，第三人称概括态度/距离感；勿编造核心性格档案未给出的硬事实。）"
            .to_string()
    } else {
        prev
    };
    format!(
        r#"你是「可变性格档案」编辑助手（系统任务，不是角色扮演）。

【职责】根据本回合对话，产出更新后的「可变性格档案」正文：只描述**与用户相处中形成**的态度、信任、习惯、心结等；让角色能**抓住自己在关系里的有限变化**。
【铁律·核心档案】「核心性格档案」由用户与创作者维护。你**不得**改写、复述替代、或输出对核心档案的补丁/新版本；**禁止**把核心内容混入你的输出。你的输出**仅**为可变档案正文。
【创作者权限】创作者只能通过配置里的数值调节本轮「允许改动的胆子」，**不能**手写本条档案；你也不要从配置里捏造具体剧情，只依据对话事实与已有档案修订。
【配置】event_impact_factor={eif:.3}（事件放大）、max_change_per_event={mce:.4}（单轮步长语义）、max_total_change={mtc:.3}（长期漂移上限的提醒，勿与核心冲突）。
【单轮步长语义】{step}

【角色】{name}

【核心性格档案】（只读锚点）
{core}

【当前可变性格档案】（在此基础上增删改）
{prev_block}

【本回合】
用户：{um}
角色回复：{br}
用户情绪标签：{emo}
事件类型：{evt:?}
归一影响（已乘运行时事件倍率）：{imp:.3}

【输出】只输出更新后的可变性格档案正文。禁止 Markdown 代码围栏、禁止标题、禁止元解释、禁止与用户的对话。中文。"#,
        eif = input.evolution.event_impact_factor,
        mce = input.evolution.max_change_per_event,
        mtc = input.evolution.max_total_change,
        name = input.role_name,
        core = if core.trim().is_empty() {
            "（核心性格档案未写，仅依据对话与已有可变档案谨慎修订。）".to_string()
        } else {
            core
        },
        prev_block = prev_block,
        um = um,
        br = br,
        emo = input.user_emotion,
        evt = input.event_type,
        imp = input.impact_scaled.clamp(-1.0, 1.0),
        step = step,
    )
}

pub fn strip_wrapping_fences(text: &str) -> String {
    let t = text.trim();
    if !t.starts_with("```") {
        return t.to_string();
    }
    let mut inner = t[3..].trim_start();
    if let Some(pos) = inner.rfind("```") {
        inner = inner[..pos].trim_end();
    }
    let inner = inner.trim();
    if let Some(i) = inner.find('\n') {
        let first = inner[..i].trim();
        if !first.chars().any(|c| c.is_whitespace())
            && first.chars().count() < 24
            && !first.is_empty()
        {
            return inner[i + 1..].trim().to_string();
        }
    }
    inner.to_string()
}

pub async fn evolve_mutable_personality_with_llm(
    llm: &Arc<dyn LlmClient>,
    model: &str,
    input: MutableEvolutionInput<'_>,
) -> Result<String> {
    let prompt = build_prompt(&input);
    let raw = llm.generate(model, &prompt).await?;
    let cleaned = strip_wrapping_fences(&raw);
    if cleaned.is_empty() {
        return Ok(input.prev_mutable.to_string());
    }
    Ok(trim_mutable_storage(&cleaned))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::llm::MockLlmClient;

    #[test]
    fn strip_fences() {
        assert_eq!(strip_wrapping_fences("```\nhello\n```"), "hello");
    }

    #[tokio::test]
    async fn evolve_uses_mock_llm() {
        let llm: Arc<dyn LlmClient> = Arc::new(MockLlmClient {
            reply: "更信任对方，语气变软。".into(),
        });
        let evo = EvolutionConfig::default();
        let out = evolve_mutable_personality_with_llm(
            &llm,
            "m",
            MutableEvolutionInput {
                role_name: "R",
                core_personality: "内向",
                prev_mutable: "",
                user_message: "谢谢",
                bot_reply: "嗯",
                user_emotion: "happy",
                event_type: &EventType::Praise,
                impact_scaled: 0.3,
                evolution: &evo,
            },
        )
        .await
        .unwrap();
        assert!(out.contains("信任") || out.contains("软"));
    }
}

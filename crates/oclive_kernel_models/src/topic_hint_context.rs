//! 话题提示轻量上下文：`top_topic_hint` 仅依赖本结构中的字段，不引用宿主完整 `Role`。

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::role_config::MemoryConfig;

/// 运行时装配与设施 crate 共用的「话题提示」输入（**不**含完整 `Role`）。
#[derive(Debug, Clone, Copy, Default)]
pub struct TopicHintContext<'a> {
    /// 与 [`MemoryConfig::topic_weights`] 相同：`scene_id` → 话题名 → 权重。
    pub topic_weights: Option<&'a HashMap<String, HashMap<String, f64>>>,
    /// 预留：对话摘要（当前选取逻辑未使用，保持 `None` 与历史行为一致）。
    pub dialogue_summary: Option<&'a str>,
    /// 预留：最近对话摘录（当前选取逻辑未使用）。
    pub recent_dialog: Option<&'a str>,
}

impl<'a> TopicHintContext<'a> {
    #[must_use]
    pub fn from_memory_config(mc: Option<&'a MemoryConfig>) -> Self {
        Self {
            topic_weights: mc.map(|m| &m.topic_weights),
            dialogue_summary: None,
            recent_dialog: None,
        }
    }

    /// 选取当前场景下权重最高的话题名（与历史 `PromptBuilder::top_topic_hint(&Role, …)` 行为一致）。
    #[must_use]
    pub fn top_topic_name_for_scene(&self, scene_id: &str) -> Option<String> {
        let tw = self.topic_weights?;
        let scene_map = tw.get(scene_id)?;
        scene_map
            .iter()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(name, _)| name.clone())
    }
}

/// JSON-RPC `prompt.top_topic_hint` 等场景的拥有型快照。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TopicHintContextSnapshot {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topic_weights: Option<HashMap<String, HashMap<String, f64>>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dialogue_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recent_dialog: Option<String>,
}

impl TopicHintContextSnapshot {
    #[must_use]
    pub fn from_borrowed(ctx: &TopicHintContext<'_>) -> Self {
        Self {
            topic_weights: ctx.topic_weights.cloned(),
            dialogue_summary: ctx.dialogue_summary.map(str::to_string),
            recent_dialog: ctx.recent_dialog.map(str::to_string),
        }
    }

    /// 将快照转为借用视图，供 builtin / 回退路径调用 `top_topic_name_for_scene`。
    #[must_use]
    pub fn as_borrowed(&self) -> TopicHintContext<'_> {
        TopicHintContext {
            topic_weights: self.topic_weights.as_ref(),
            dialogue_summary: self.dialogue_summary.as_deref(),
            recent_dialog: self.recent_dialog.as_deref(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_max_weight_topic() {
        let mut home = HashMap::new();
        home.insert("日常".to_string(), 0.3);
        home.insert("学习".to_string(), 0.9);
        let mut weights = HashMap::new();
        weights.insert("家".to_string(), home);
        let ctx = TopicHintContext {
            topic_weights: Some(&weights),
            dialogue_summary: None,
            recent_dialog: None,
        };
        assert_eq!(
            ctx.top_topic_name_for_scene("家").as_deref(),
            Some("学习")
        );
    }

    #[test]
    fn missing_scene_returns_none() {
        let ctx = TopicHintContext::default();
        assert!(ctx.top_topic_name_for_scene("x").is_none());
    }

    #[test]
    fn snapshot_roundtrip_borrowed() {
        let mut m = HashMap::new();
        m.insert("a".to_string(), 1.0);
        let mut outer = HashMap::new();
        outer.insert("s".to_string(), m);
        let ctx = TopicHintContext {
            topic_weights: Some(&outer),
            dialogue_summary: Some("sum"),
            recent_dialog: None,
        };
        let snap = TopicHintContextSnapshot::from_borrowed(&ctx);
        assert_eq!(snap.as_borrowed().top_topic_name_for_scene("s").as_deref(), Some("a"));
        assert_eq!(snap.as_borrowed().dialogue_summary, Some("sum"));
    }
}

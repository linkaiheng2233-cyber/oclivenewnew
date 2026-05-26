//! 专家路由 JSON（`expert_routing` 或 `blueprint/includes/expert_routing.json`）。

use std::path::Path;

use serde::{Deserialize, Serialize};

/// 默认卫星路径（相对角色包根）。
pub const DEFAULT_EXPERT_ROUTING_PATH: &str = "blueprint/includes/expert_routing.json";

/// 专家流程失败时的降级策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExpertFallback {
    #[default]
    Skip,
    RetryWithDefault,
}

impl ExpertFallback {
    #[must_use]
    pub fn from_str_lossy(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "retry_with_default" => Self::RetryWithDefault,
            _ => Self::Skip,
        }
    }
}

/// 消息长度范围（`message_length` 或兼容顶层 `min/max_message_length`）。
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct MessageLengthRange {
    #[serde(default)]
    pub min: Option<u32>,
    #[serde(default)]
    pub max: Option<u32>,
}

/// 时间段窗口（`HH:MM` 24h，含端点；跨午夜时 `after > before` 表示夜间区间）。
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct TimeOfDayWindow {
    #[serde(default)]
    pub after: Option<String>,
    #[serde(default)]
    pub before: Option<String>,
}

/// 专家路由触发条件（全部**已设置**的字段须同时满足）。
pub type TriggerCondition = ExpertTrigger;

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct ExpertTrigger {
    #[serde(default, alias = "scene_ids")]
    pub scenes: Option<Vec<String>>,
    #[serde(default)]
    pub keywords: Option<Vec<String>>,
    #[serde(default)]
    pub user_emotion: Option<Vec<String>>,
    #[serde(default)]
    pub message_length: Option<MessageLengthRange>,
    /// 兼容旧字段：映射到 `message_length.min`。
    #[serde(default)]
    pub min_message_length: Option<u32>,
    /// 兼容旧字段：映射到 `message_length.max`。
    #[serde(default)]
    pub max_message_length: Option<u32>,
    #[serde(default)]
    pub time_of_day: Option<TimeOfDayWindow>,
    #[serde(default)]
    pub user_relation: Option<Vec<String>>,
}

impl TriggerCondition {
    /// 是否满足本回合匹配上下文。
    #[must_use]
    pub fn matches(&self, ctx: &ExpertMatchContext) -> bool {
        trigger_matches(self, ctx)
    }
}

impl ExpertTrigger {
    #[must_use]
    pub fn effective_message_length(&self) -> MessageLengthRange {
        let mut r = self.message_length.clone().unwrap_or_default();
        if r.min.is_none() {
            r.min = self.min_message_length;
        }
        if r.max.is_none() {
            r.max = self.max_message_length;
        }
        r
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct ExpertStepParams(pub serde_json::Value);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ExpertRouteStep {
    pub action: String,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub params: Option<ExpertStepParams>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ExpertRoute {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// 数值越大优先级越高；相同则按 `routes` 数组顺序取首个匹配。
    #[serde(default)]
    pub priority: Option<i32>,
    #[serde(default)]
    pub trigger: ExpertTrigger,
    #[serde(default)]
    pub steps: Vec<ExpertRouteStep>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct ExpertRoutingDoc {
    #[serde(default)]
    pub routing_path: Option<String>,
    #[serde(default)]
    pub fallback: Option<String>,
    #[serde(default)]
    pub routes: Vec<ExpertRoute>,
}

impl ExpertRoutingDoc {
    #[must_use]
    pub fn fallback_mode(&self) -> ExpertFallback {
        self.fallback
            .as_deref()
            .map(ExpertFallback::from_str_lossy)
            .unwrap_or_default()
    }
}

/// 校验专家路由文档（结构 + 步骤 action 白名单）。
///
/// # Errors
pub fn validate_expert_routing_doc(doc: &ExpertRoutingDoc) -> Result<(), Vec<String>> {
    let mut errs = Vec::new();
    for (idx, route) in doc.routes.iter().enumerate() {
        let label = route
            .id
            .as_deref()
            .filter(|s| !s.is_empty())
            .map(|s| format!("routes[{idx}] id={s}"))
            .unwrap_or_else(|| format!("routes[{idx}]"));
        for (sidx, step) in route.steps.iter().enumerate() {
            if let Err(e) = crate::expert_actions::validate_expert_step_action(&step.action) {
                errs.push(format!("{label} steps[{sidx}]: {e}"));
            }
        }
    }
    if errs.is_empty() {
        Ok(())
    } else {
        Err(errs)
    }
}

/// 从角色包目录读取专家路由（文件缺失返回 `None`）。
pub fn load_expert_routing_from_role_dir(role_dir: &Path) -> Option<ExpertRoutingDoc> {
    let path = role_dir.join(DEFAULT_EXPERT_ROUTING_PATH);
    if !path.is_file() {
        return None;
    }
    let raw = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&raw).ok()
}

/// 匹配上下文（宿主在 `TurnContext` / 实验步上下文上填充）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpertMatchContext {
    pub scene_id: String,
    pub user_message: String,
    pub user_emotion: Option<String>,
    pub user_relation: Option<String>,
    /// 虚拟时间毫秒；0 表示未设置（`time_of_day` 用墙钟）。
    pub virtual_time_ms: i64,
    /// 墙钟 `(hour, minute)`，用于无虚拟时间时的 `time_of_day`。
    pub wall_clock_hour_minute: (u32, u32),
}

/// 触发条件是否全部满足。
#[must_use]
pub fn trigger_matches(trigger: &ExpertTrigger, ctx: &ExpertMatchContext) -> bool {
    if let Some(ref scenes) = trigger.scenes {
        if !scenes.is_empty() && !scenes.iter().any(|s| s == &ctx.scene_id) {
            return false;
        }
    }
    if let Some(ref kws) = trigger.keywords {
        if !kws.is_empty() {
            let lower = ctx.user_message.to_ascii_lowercase();
            if !kws.iter().any(|k| {
                let needle = k.trim().to_ascii_lowercase();
                !needle.is_empty() && lower.contains(needle.as_str())
            }) {
                return false;
            }
        }
    }
    if let Some(ref emotions) = trigger.user_emotion {
        if !emotions.is_empty() {
            let cur = ctx
                .user_emotion
                .as_deref()
                .unwrap_or("")
                .trim()
                .to_ascii_lowercase();
            if !emotions.iter().any(|e| e.trim().eq_ignore_ascii_case(cur.as_str())) {
                return false;
            }
        }
    }
    let len_range = trigger.effective_message_length();
    let msg_len = ctx.user_message.chars().count() as u32;
    if let Some(min) = len_range.min {
        if msg_len < min {
            return false;
        }
    }
    if let Some(max) = len_range.max {
        if msg_len > max {
            return false;
        }
    }
    if let Some(ref tod) = trigger.time_of_day {
        if !time_of_day_matches(tod, ctx) {
            return false;
        }
    }
    if let Some(ref rels) = trigger.user_relation {
        if !rels.is_empty() {
            let cur = ctx
                .user_relation
                .as_deref()
                .unwrap_or("")
                .trim()
                .to_ascii_lowercase();
            if !rels.iter().any(|r| r.trim().eq_ignore_ascii_case(cur.as_str())) {
                return false;
            }
        }
    }
    true
}

fn time_of_day_matches(window: &TimeOfDayWindow, ctx: &ExpertMatchContext) -> bool {
    let (h, m) = if ctx.virtual_time_ms > 0 {
        let secs = (ctx.virtual_time_ms / 1000) % 86_400;
        let h = (secs / 3600) as u32;
        let m = ((secs % 3600) / 60) as u32;
        (h, m)
    } else {
        ctx.wall_clock_hour_minute
    };
    let cur = h * 60 + m;
    let after = window.after.as_deref().and_then(parse_hhmm);
    let before = window.before.as_deref().and_then(parse_hhmm);
    match (after, before) {
        (None, None) => true,
        (Some(a), None) => cur >= a,
        (None, Some(b)) => cur <= b,
        (Some(a), Some(b)) if a <= b => cur >= a && cur <= b,
        (Some(a), Some(b)) => cur >= a || cur <= b,
    }
}

fn parse_hhmm(s: &str) -> Option<u32> {
    let parts: Vec<&str> = s.trim().split(':').collect();
    if parts.len() != 2 {
        return None;
    }
    let h: u32 = parts[0].parse().ok()?;
    let m: u32 = parts[1].parse().ok()?;
    if h >= 24 || m >= 60 {
        return None;
    }
    Some(h * 60 + m)
}

/// 按 `priority` 降序选取**首个**完全匹配的路由（同优先级保留定义顺序）。
#[must_use]
pub fn select_expert_route<'a>(
    doc: &'a ExpertRoutingDoc,
    ctx: &ExpertMatchContext,
) -> Option<&'a ExpertRoute> {
    let mut best: Option<(&ExpertRoute, i32, usize)> = None;
    for (idx, route) in doc.routes.iter().enumerate() {
        if !route.enabled || !trigger_matches(&route.trigger, ctx) {
            continue;
        }
        let pri = route.priority.unwrap_or(0);
        let replace = match best {
            None => true,
            Some((_, bp, bi)) => pri > bp || (pri == bp && idx < bi),
        };
        if replace {
            best = Some((route, pri, idx));
        }
    }
    best.map(|(r, _, _)| r)
}

/// 兼容旧 API。
#[must_use]
pub fn match_expert_route<'a>(
    doc: &'a ExpertRoutingDoc,
    scene_id: &str,
    user_message: &str,
) -> Option<&'a ExpertRoute> {
    let ctx = ExpertMatchContext {
        scene_id: scene_id.into(),
        user_message: user_message.into(),
        user_emotion: None,
        user_relation: None,
        virtual_time_ms: 0,
        wall_clock_hour_minute: (12, 0),
    };
    select_expert_route(doc, &ctx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_route_by_keyword() {
        let doc = ExpertRoutingDoc {
            routes: vec![ExpertRoute {
                id: Some("k".into()),
                enabled: true,
                priority: None,
                trigger: ExpertTrigger {
                    keywords: Some(vec!["专家".into()]),
                    ..Default::default()
                },
                steps: vec![],
            }],
            ..Default::default()
        };
        let ctx = ExpertMatchContext {
            scene_id: "cafe".into(),
            user_message: "请专家帮忙".into(),
            user_emotion: None,
            user_relation: None,
            virtual_time_ms: 0,
            wall_clock_hour_minute: (12, 0),
        };
        assert!(select_expert_route(&doc, &ctx).is_some());
        assert!(match_expert_route(&doc, "cafe", "你好").is_none());
    }

    #[test]
    fn priority_picks_higher_route() {
        let doc = ExpertRoutingDoc {
            routes: vec![
                ExpertRoute {
                    id: Some("low".into()),
                    enabled: true,
                    priority: Some(1),
                    trigger: ExpertTrigger {
                        keywords: Some(vec!["x".into()]),
                        ..Default::default()
                    },
                    steps: vec![],
                },
                ExpertRoute {
                    id: Some("high".into()),
                    enabled: true,
                    priority: Some(10),
                    trigger: ExpertTrigger {
                        keywords: Some(vec!["x".into()]),
                        ..Default::default()
                    },
                    steps: vec![],
                },
            ],
            ..Default::default()
        };
        let ctx = ExpertMatchContext {
            scene_id: "s".into(),
            user_message: "x".into(),
            user_emotion: None,
            user_relation: None,
            virtual_time_ms: 0,
            wall_clock_hour_minute: (12, 0),
        };
        assert_eq!(
            select_expert_route(&doc, &ctx).and_then(|r| r.id.as_deref()),
            Some("high")
        );
    }

    #[test]
    fn scene_and_emotion_and() {
        let doc = ExpertRoutingDoc {
            routes: vec![ExpertRoute {
                enabled: true,
                priority: None,
                trigger: ExpertTrigger {
                    scenes: Some(vec!["park".into()]),
                    user_emotion: Some(vec!["happy".into()]),
                    ..Default::default()
                },
                steps: vec![],
                id: None,
            }],
            ..Default::default()
        };
        let ok = ExpertMatchContext {
            scene_id: "park".into(),
            user_message: "hi".into(),
            user_emotion: Some("Happy".into()),
            user_relation: None,
            virtual_time_ms: 0,
            wall_clock_hour_minute: (12, 0),
        };
        let bad = ExpertMatchContext {
            scene_id: "cafe".into(),
            user_message: "hi".into(),
            user_emotion: Some("Happy".into()),
            user_relation: None,
            virtual_time_ms: 0,
            wall_clock_hour_minute: (12, 0),
        };
        assert!(select_expert_route(&doc, &ok).is_some());
        assert!(select_expert_route(&doc, &bad).is_none());
    }
}

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

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ExpertTrigger {
    #[serde(default)]
    pub scene_ids: Option<Vec<String>>,
    #[serde(default)]
    pub keywords: Option<Vec<String>>,
    #[serde(default)]
    pub min_message_length: Option<u32>,
    #[serde(default)]
    pub max_message_length: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExpertRouteStep {
    pub action: String,
    #[serde(default)]
    pub depends_on: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExpertRoute {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub trigger: ExpertTrigger,
    #[serde(default)]
    pub steps: Vec<ExpertRouteStep>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
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

/// 从角色包目录读取专家路由（文件缺失返回 `None`）。
pub fn load_expert_routing_from_role_dir(role_dir: &Path) -> Option<ExpertRoutingDoc> {
    let path = role_dir.join(DEFAULT_EXPERT_ROUTING_PATH);
    if !path.is_file() {
        return None;
    }
    let raw = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&raw).ok()
}

/// 给定场景与消息，返回第一个启用的匹配路由。
#[must_use]
pub fn match_expert_route<'a>(
    doc: &'a ExpertRoutingDoc,
    scene_id: &str,
    user_message: &str,
) -> Option<&'a ExpertRoute> {
    let msg_len = user_message.chars().count() as u32;
    doc.routes.iter().find(|r| {
        if !r.enabled {
            return false;
        }
        route_matches(&r.trigger, scene_id, user_message, msg_len)
    })
}

fn route_matches(trigger: &ExpertTrigger, scene_id: &str, user_message: &str, msg_len: u32) -> bool {
    if let Some(ref scenes) = trigger.scene_ids {
        if !scenes.is_empty() && !scenes.iter().any(|s| s == scene_id) {
            return false;
        }
    }
    if let Some(min) = trigger.min_message_length {
        if msg_len < min {
            return false;
        }
    }
    if let Some(max) = trigger.max_message_length {
        if msg_len > max {
            return false;
        }
    }
    if let Some(ref kws) = trigger.keywords {
        if !kws.is_empty() {
            let lower = user_message.to_ascii_lowercase();
            if !kws.iter().any(|k| {
                let needle = k.trim().to_ascii_lowercase();
                !needle.is_empty() && lower.contains(needle.as_str())
            }) {
                return false;
            }
        }
    }
    true
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
                trigger: ExpertTrigger {
                    keywords: Some(vec!["专家".into()]),
                    ..Default::default()
                },
                steps: vec![],
            }],
            ..Default::default()
        };
        assert!(match_expert_route(&doc, "cafe", "请专家帮忙").is_some());
        assert!(match_expert_route(&doc, "cafe", "你好").is_none());
    }
}

//! 双核 `pipeline.*.steps[].action` 解析。

/// 实验/稳定 pipeline 支持的专家设施 action。
pub const PIPELINE_ACTION_EXPERT_INVOKE: &str = "slot.expert.invoke";

/// 解析后的 pipeline action。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PipelineActionKind {
    /// `slot.<registry_key>.<method>`
    Slot {
        registry_key: String,
        method: String,
    },
    /// `slot.expert.invoke`（不占用 `slot_registry` 键）
    ExpertInvoke,
}

/// 解析 `action` 字符串。
///
/// # Errors
///
/// 格式非法时返回说明字符串。
pub fn parse_pipeline_action_kind(action: &str) -> Result<PipelineActionKind, String> {
    let trimmed = action.trim();
    if trimmed == PIPELINE_ACTION_EXPERT_INVOKE {
        return Ok(PipelineActionKind::ExpertInvoke);
    }
    let (registry_key, method) = parse_slot_action(trimmed)?;
    Ok(PipelineActionKind::Slot {
        registry_key,
        method,
    })
}

/// 兼容旧 API：`slot.<key>.<method>`；`slot.expert.invoke` 映射为 `("expert", "invoke")`。
///
/// # Errors
pub fn parse_pipeline_action(action: &str) -> Result<(String, String), String> {
    match parse_pipeline_action_kind(action)? {
        PipelineActionKind::ExpertInvoke => Ok(("expert".into(), "invoke".into())),
        PipelineActionKind::Slot {
            registry_key,
            method,
        } => Ok((registry_key, method)),
    }
}

fn parse_slot_action(action: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = action.split('.').collect();
    if parts.len() < 3 || parts[0] != "slot" {
        return Err(format!(
            "action「{action}」须为 slot.<registry_key>.<method> 或 {PIPELINE_ACTION_EXPERT_INVOKE}"
        ));
    }
    let key = parts[1].trim();
    if key.is_empty() {
        return Err(format!("action「{action}」缺少 registry_key"));
    }
    if key == "expert" {
        return Err(format!(
            "action「{action}」请使用 {PIPELINE_ACTION_EXPERT_INVOKE} 调用专家设施"
        ));
    }
    let method = parts[2..].join(".");
    if method.is_empty() {
        return Err(format!("action「{action}」缺少 method"));
    }
    Ok((key.to_string(), method))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expert_invoke_parses() {
        assert_eq!(
            parse_pipeline_action_kind(PIPELINE_ACTION_EXPERT_INVOKE).unwrap(),
            PipelineActionKind::ExpertInvoke
        );
    }

    #[test]
    fn slot_action_parses() {
        let k = parse_pipeline_action_kind("slot.llm_main.generate").unwrap();
        assert_eq!(
            k,
            PipelineActionKind::Slot {
                registry_key: "llm_main".into(),
                method: "generate".into()
            }
        );
    }

    #[test]
    fn rejects_slot_expert_dot_invoke_alias() {
        assert!(parse_pipeline_action_kind("slot.expert.invoke.extra").is_err());
    }
}

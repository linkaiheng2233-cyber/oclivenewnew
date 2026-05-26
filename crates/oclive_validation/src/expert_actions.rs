//! 专家设施步骤 `action` 解析与校验。

/// 专家设施专用 action（非 `slot_registry` 键）。
pub const EXPERT_ACTION_PERSONALITY_ADJUST: &str = "slot.personality.adjust";
pub const EXPERT_ACTION_PROMPT_ENHANCE: &str = "slot.prompt_enhance.apply";
pub const EXPERT_ACTION_MEMORY_INJECT: &str = "slot.memory.inject";
pub const EXPERT_ACTION_LORA_APPLY: &str = "slot.lora.apply";
pub const EXPERT_ACTION_EXPERT_FALLBACK: &str = "slot.expert.fallback";

const FACILITY_ACTIONS: &[&str] = &[
    EXPERT_ACTION_PERSONALITY_ADJUST,
    EXPERT_ACTION_PROMPT_ENHANCE,
    EXPERT_ACTION_MEMORY_INJECT,
    EXPERT_ACTION_LORA_APPLY,
    EXPERT_ACTION_EXPERT_FALLBACK,
];

/// 专家子步骤 action 种类。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpertStepActionKind {
    Slot {
        registry_key: String,
        method: String,
    },
    PersonalityAdjust,
    PromptEnhanceApply,
    MemoryInject,
    LoraApply,
    ExpertFallback,
}

/// 解析专家路由单步 `action`。
///
/// # Errors
pub fn parse_expert_step_action(action: &str) -> Result<ExpertStepActionKind, String> {
    let trimmed = action.trim();
    match trimmed {
        EXPERT_ACTION_PERSONALITY_ADJUST => Ok(ExpertStepActionKind::PersonalityAdjust),
        EXPERT_ACTION_PROMPT_ENHANCE => Ok(ExpertStepActionKind::PromptEnhanceApply),
        EXPERT_ACTION_MEMORY_INJECT => Ok(ExpertStepActionKind::MemoryInject),
        EXPERT_ACTION_LORA_APPLY => Ok(ExpertStepActionKind::LoraApply),
        EXPERT_ACTION_EXPERT_FALLBACK => Ok(ExpertStepActionKind::ExpertFallback),
        _ => {
            let (key, method) = parse_slot_step(trimmed)?;
            Ok(ExpertStepActionKind::Slot {
                registry_key: key,
                method,
            })
        }
    }
}

fn parse_slot_step(action: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = action.split('.').collect();
    if parts.len() < 3 || parts[0] != "slot" {
        return Err(format!(
            "action「{action}」须为 slot.<registry_key>.<method> 或专家设施 action"
        ));
    }
    let key = parts[1].trim();
    if key.is_empty() || key == "expert" {
        return Err(format!("action「{action}」非法 registry_key"));
    }
    let reserved = [
        "personality",
        "prompt_enhance",
        "memory",
        "lora",
    ];
    if reserved.contains(&key) {
        return Err(format!("action「{action}」请使用专家设施专用 action"));
    }
    let method = parts[2..].join(".");
    if method.is_empty() {
        return Err(format!("action「{action}」缺少 method"));
    }
    Ok((key.to_string(), method))
}

/// 校验步骤 action 是否合法（slot 或设施）。
///
/// # Errors
pub fn validate_expert_step_action(action: &str) -> Result<(), String> {
    parse_expert_step_action(action).map(|_| ())
}

/// 是否为已知专家设施 action 常量。
#[must_use]
pub fn is_facility_action(action: &str) -> bool {
    FACILITY_ACTIONS.contains(&action.trim())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn facility_actions_parse() {
        assert_eq!(
            parse_expert_step_action(EXPERT_ACTION_MEMORY_INJECT).unwrap(),
            ExpertStepActionKind::MemoryInject
        );
    }

    #[test]
    fn slot_step_parses() {
        let k = parse_expert_step_action("slot.llm_main.generate").unwrap();
        assert_eq!(
            k,
            ExpertStepActionKind::Slot {
                registry_key: "llm_main".into(),
                method: "generate".into()
            }
        );
    }
}

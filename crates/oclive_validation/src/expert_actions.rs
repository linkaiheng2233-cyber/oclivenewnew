//! Expert facility step `action` parsing and validation.

/// Expert-facility-specific actions (not `slot_registry` keys).
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

/// Kinds of expert sub-step actions.
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

/// Parse a single expert routing step `action`.
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

/// Validate whether a step action is valid (slot or facility).
///
/// # Errors
pub fn validate_expert_step_action(action: &str) -> Result<(), String> {
    parse_expert_step_action(action).map(|_| ())
}

/// Whether this is a known expert facility action constant.
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

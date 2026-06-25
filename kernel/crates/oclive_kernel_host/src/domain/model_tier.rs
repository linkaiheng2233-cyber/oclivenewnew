//! Ollama model tier heuristic for Wave D persona capsule routing (编排行 · 非六槽).

use crate::domain::host_profile::{HostProfile, TurnThinkingProfile};
use crate::domain::turn_thinking::TurnThinkingMode;
use crate::models::Role;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelTier {
    Small,
    Large,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersonaSource {
    FullCore,
    DeepCapsule,
}

#[must_use]
pub fn resolve_model_tier(ollama_model: &str) -> ModelTier {
    let lower = ollama_model.to_ascii_lowercase();
    if lower.contains("70b")
        || lower.contains("72b")
        || lower.contains("34b")
        || lower.contains("65b")
        || lower.contains("405b")
    {
        return ModelTier::Large;
    }
    if lower.contains("7b")
        || lower.contains("8b")
        || lower.contains("13b")
        || lower.contains("9b")
        || lower.contains("12b")
    {
        return ModelTier::Small;
    }
    ModelTier::Small
}

#[must_use]
pub fn resolve_persona_source(
    tier: ModelTier,
    mode: TurnThinkingMode,
    role: &Role,
    host: &HostProfile,
) -> PersonaSource {
    if mode != TurnThinkingMode::Deep || tier != ModelTier::Small {
        return PersonaSource::FullCore;
    }
    let has_capsule = role
        .deep_capsule
        .as_ref()
        .is_some_and(|s| !s.trim().is_empty());
    if !has_capsule {
        return PersonaSource::FullCore;
    }
    let enabled = deep_capsule_effective_enabled(role.deep_capsule_enabled, &host.turn_thinking);
    if enabled {
        PersonaSource::DeepCapsule
    } else {
        PersonaSource::FullCore
    }
}

#[must_use]
pub fn persona_override_for_source(role: &Role, source: PersonaSource) -> Option<&str> {
    match source {
        PersonaSource::DeepCapsule => role.deep_capsule.as_deref(),
        PersonaSource::FullCore => None,
    }
}

fn deep_capsule_effective_enabled(role_enabled: bool, host: &TurnThinkingProfile) -> bool {
    match host.deep_capsule {
        Some(true) => true,
        Some(false) => false,
        None => role_enabled,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::turn_thinking::TurnThinkingMode;

    fn role_with_capsule(enabled: bool) -> Role {
        Role {
            deep_capsule_enabled: enabled,
            deep_capsule: Some("短人设".into()),
            ..Role::default()
        }
    }

    #[test]
    fn tier_small_for_7b() {
        assert_eq!(resolve_model_tier("qwen2.5:7b"), ModelTier::Small);
    }

    #[test]
    fn tier_large_for_70b() {
        assert_eq!(resolve_model_tier("llama3.1:70b"), ModelTier::Large);
    }

    #[test]
    fn capsule_only_small_deep_enabled() {
        let role = role_with_capsule(true);
        let host = HostProfile::default();
        assert_eq!(
            resolve_persona_source(ModelTier::Small, TurnThinkingMode::Deep, &role, &host),
            PersonaSource::DeepCapsule
        );
        assert_eq!(
            resolve_persona_source(ModelTier::Large, TurnThinkingMode::Deep, &role, &host),
            PersonaSource::FullCore
        );
        assert_eq!(
            resolve_persona_source(ModelTier::Small, TurnThinkingMode::Fast, &role, &host),
            PersonaSource::FullCore
        );
    }

    #[test]
    fn host_deep_capsule_false_blocks() {
        let role = role_with_capsule(true);
        let mut host = HostProfile::default();
        host.turn_thinking.deep_capsule = Some(false);
        assert_eq!(
            resolve_persona_source(ModelTier::Small, TurnThinkingMode::Deep, &role, &host),
            PersonaSource::FullCore
        );
    }
}

//! Interaction mode (immersive vs pure chat): parsing, validation, and API strings in one place.

/// Aligned with `role_runtime.interaction_mode`, DTOs, and `settings.json` conventions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionMode {
    Immersive,
    PureChat,
}

impl InteractionMode {
    pub const IMMERSIVE: &'static str = "immersive";
    pub const PURE_CHAT: &'static str = "pure_chat";

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Immersive => Self::IMMERSIVE,
            Self::PureChat => Self::PURE_CHAT,
        }
    }

    /// Any source (DB / legacy data) → canonical value; unknown or empty defaults to pure_chat.
    #[must_use]
    pub fn normalize(raw: Option<&str>) -> Self {
        match raw.map(str::trim).filter(|s| !s.is_empty()) {
            Some(s) if s == Self::PURE_CHAT => Self::PureChat,
            Some(s) if s == Self::IMMERSIVE => Self::Immersive,
            _ => Self::PureChat,
        }
    }

    #[must_use]
    pub const fn is_immersive(self) -> bool {
        matches!(self, Self::Immersive)
    }
    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    /// Validates the optional role pack `settings.json` field.
    pub fn validate_optional_pack_field(raw: Option<&str>) -> Result<(), String> {
        if let Some(s) = raw {
            let t = s.trim();
            if !t.is_empty() && Self::parse_exact(t).is_none() {
                return Err(format!(
                    "角色包 settings：interaction_mode 须为 {} 或 {}（当前为 {}）",
                    Self::IMMERSIVE,
                    Self::PURE_CHAT,
                    s
                ));
            }
        }
        Ok(())
    }

    fn parse_exact(raw: &str) -> Option<Self> {
        match raw.trim() {
            t if t == Self::IMMERSIVE => Some(Self::Immersive),
            t if t == Self::PURE_CHAT => Some(Self::PureChat),
            _ => None,
        }
    }

    /// For API `interaction_mode_pack_default`: pass through only valid values unchanged.
    #[must_use]
    pub fn pack_default_for_api(raw: Option<&str>) -> Option<String> {
        raw.and_then(|s| Self::parse_exact(s).map(|m| m.as_str().to_string()))
    }

    /// Distro/pack hint for docs and non-DB defaults; runtime first-run seed uses [`InteractionMode::PureChat`].
    #[must_use]
    pub fn seed_default(distro: Option<&str>, pack: Option<&str>) -> Self {
        if let Some(raw) = distro {
            if let Some(m) = Self::parse_exact(raw) {
                return m;
            }
        }
        if let Some(raw) = pack {
            if let Some(m) = Self::parse_exact(raw) {
                return m;
            }
        }
        Self::PureChat
    }
}

#[cfg(test)]
mod tests {
    use super::InteractionMode;

    #[test]
    fn normalize_defaults_unknown_to_pure_chat() {
        assert!(!InteractionMode::normalize(None).is_immersive());
        assert!(!InteractionMode::normalize(Some("")).is_immersive());
        assert!(!InteractionMode::normalize(Some("  ")).is_immersive());
        assert!(!InteractionMode::normalize(Some("other")).is_immersive());
    }

    #[test]
    fn normalize_accepts_canonical() {
        assert!(!InteractionMode::normalize(Some("pure_chat")).is_immersive());
        assert!(InteractionMode::normalize(Some("immersive")).is_immersive());
    }

    #[test]
    fn pack_default_for_api_filters_invalid() {
        assert_eq!(InteractionMode::pack_default_for_api(None), None);
        assert_eq!(
            InteractionMode::pack_default_for_api(Some("pure_chat")),
            Some("pure_chat".to_string())
        );
        assert_eq!(InteractionMode::pack_default_for_api(Some("nope")), None);
    }

    #[test]
    fn seed_default_prefers_distro_then_pack_then_pure_chat() {
        assert_eq!(
            InteractionMode::seed_default(Some("immersive"), Some("pure_chat")),
            InteractionMode::Immersive
        );
        assert_eq!(
            InteractionMode::seed_default(Some("nope"), Some("pure_chat")),
            InteractionMode::PureChat
        );
        assert_eq!(
            InteractionMode::seed_default(None, Some("immersive")),
            InteractionMode::Immersive
        );
        assert_eq!(
            InteractionMode::seed_default(None, None),
            InteractionMode::PureChat
        );
    }
}

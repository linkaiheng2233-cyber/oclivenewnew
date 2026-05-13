//! 交互模式（沉浸 vs 纯聊）：解析、校验与 API 字符串，单点维护。

/// 与 `role_runtime.interaction_mode`、DTO、`settings.json` 约定一致。
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

    /// 任意来源（DB / 旧数据）→ 规范值；未知或空视为沉浸。
    #[must_use]
    pub fn normalize(raw: Option<&str>) -> Self {
        match raw.map(str::trim).filter(|s| !s.is_empty()) {
            Some(s) if s == Self::PURE_CHAT => Self::PureChat,
            Some(s) if s == Self::IMMERSIVE => Self::Immersive,
            _ => Self::Immersive,
        }
    }

    #[must_use]
    pub const fn is_immersive(self) -> bool {
        matches!(self, Self::Immersive)
    }

    /// 校验角色包 `settings.json` 可选字段。
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

    /// 供 API `interaction_mode_pack_default`：仅合法值原样透出。
    #[must_use]
    pub fn pack_default_for_api(raw: Option<&str>) -> Option<String> {
        raw.and_then(|s| Self::parse_exact(s).map(|m| m.as_str().to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::InteractionMode;

    #[test]
    fn normalize_defaults_unknown_to_immersive() {
        assert!(InteractionMode::normalize(None).is_immersive());
        assert!(InteractionMode::normalize(Some("")).is_immersive());
        assert!(InteractionMode::normalize(Some("  ")).is_immersive());
        assert!(InteractionMode::normalize(Some("other")).is_immersive());
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
}

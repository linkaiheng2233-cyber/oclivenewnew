//! Role pack `config.json` → `meta_action_templates` (undo/regenerate/edit/delete attitude copy).

use serde::{Deserialize, Serialize};

/// Single meta-action template (attitude line injected after storage mutation).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MetaActionTemplateEntry {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub attitude_text: String,
}

impl Default for MetaActionTemplateEntry {
    fn default() -> Self {
        Self {
            enabled: true,
            attitude_text: String::new(),
        }
    }
}

/// `config.json` → `meta_action_templates`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct RolePackMetaActionTemplatesConfig {
    #[serde(default)]
    pub undo: MetaActionTemplateEntry,
    #[serde(default)]
    pub regenerate: MetaActionTemplateEntry,
    #[serde(default)]
    pub edit: MetaActionTemplateEntry,
    #[serde(default)]
    pub delete: MetaActionTemplateEntry,
}

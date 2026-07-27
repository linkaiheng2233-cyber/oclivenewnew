//! Optional Chat Pro adult-role extension contract.
//!
//! The file lives at `roles/{role_id}/adult_extension.json`. It is intentionally
//! independent from the universal role-pack blueprint so runtimes that do not
//! support the extension can safely ignore it.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const ADULT_ROLE_EXTENSION_SCHEMA_VERSION: u32 = 1;

fn default_schema_version() -> u32 {
    ADULT_ROLE_EXTENSION_SCHEMA_VERSION
}

fn default_suggested_interval_ms() -> u64 {
    4_000
}

/// Creator pacing recommendation. Chat Pro may apply a global user override.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdultPacingConfig {
    /// `creator` uses `suggested_interval_ms`; `ai` lets the model recommend
    /// each beat's interval.
    #[serde(default = "default_pacing_mode")]
    pub mode: String,
    #[serde(default = "default_suggested_interval_ms")]
    pub suggested_interval_ms: u64,
}

fn default_pacing_mode() -> String {
    "creator".to_string()
}

impl Default for AdultPacingConfig {
    fn default() -> Self {
        Self {
            mode: default_pacing_mode(),
            suggested_interval_ms: default_suggested_interval_ms(),
        }
    }
}

/// Adult direction layered onto one ordinary scene.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdultSceneDirection {
    /// How this scene may naturally develop after the two runtime gates are on.
    #[serde(default)]
    pub direction: String,
    /// Creator guidance for action continuity and beat progression.
    #[serde(default)]
    pub action_flow: String,
    /// Optional scene-specific dialogue guidance.
    #[serde(default)]
    pub dialogue_guidance: String,
}

/// Optional role-pack extension authored by the independent R18 editor page.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdultRoleExtension {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    /// Mandatory author assertion: the fictional character is an adult.
    #[serde(default)]
    pub character_is_adult: bool,
    /// Adult-state persona delta; the base persona remains the universal SSOT.
    #[serde(default)]
    pub persona: String,
    /// Flirting / intimate dialogue style and boundaries for this character.
    #[serde(default)]
    pub dialogue_guidance: String,
    #[serde(default)]
    pub pacing: AdultPacingConfig,
    /// Keys are ordinary role-pack scene ids.
    #[serde(default)]
    pub scenes: BTreeMap<String, AdultSceneDirection>,
}

impl Default for AdultRoleExtension {
    fn default() -> Self {
        Self {
            schema_version: ADULT_ROLE_EXTENSION_SCHEMA_VERSION,
            character_is_adult: false,
            persona: String::new(),
            dialogue_guidance: String::new(),
            pacing: AdultPacingConfig::default(),
            scenes: BTreeMap::new(),
        }
    }
}

impl AdultRoleExtension {
    /// Validate the extension without imposing it on universal role packs.
    ///
    /// # Errors
    ///
    /// Returns concise field-oriented errors for creator tools and runtime load.
    pub fn validate(&self, known_scene_ids: &[String]) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if self.schema_version != ADULT_ROLE_EXTENSION_SCHEMA_VERSION {
            errors.push(format!(
                "adult_extension.schema_version must be {ADULT_ROLE_EXTENSION_SCHEMA_VERSION}"
            ));
        }
        if !self.character_is_adult {
            errors.push("adult_extension.character_is_adult must be true".to_string());
        }
        if !matches!(self.pacing.mode.trim(), "creator" | "ai") {
            errors.push("adult_extension.pacing.mode must be creator or ai".to_string());
        }
        if self.pacing.suggested_interval_ms == 0 {
            errors
                .push("adult_extension.pacing.suggested_interval_ms must be positive".to_string());
        }
        for scene_id in self.scenes.keys() {
            if scene_id.trim().is_empty() {
                errors.push("adult_extension.scenes contains an empty scene id".to_string());
            } else if !known_scene_ids.iter().any(|id| id == scene_id) {
                errors.push(format!(
                    "adult_extension.scenes.{scene_id} is not declared by the base role pack"
                ));
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_adult_and_unknown_scene() {
        let extension = AdultRoleExtension {
            scenes: BTreeMap::from([("library".into(), AdultSceneDirection::default())]),
            ..AdultRoleExtension::default()
        };
        let errors = extension.validate(&["home".into()]).expect_err("invalid");
        assert!(errors.iter().any(|e| e.contains("character_is_adult")));
        assert!(errors.iter().any(|e| e.contains("library")));
    }

    #[test]
    fn accepts_minimal_adult_extension() {
        let extension = AdultRoleExtension {
            character_is_adult: true,
            ..AdultRoleExtension::default()
        };
        assert!(extension.validate(&["home".into()]).is_ok());
    }
}

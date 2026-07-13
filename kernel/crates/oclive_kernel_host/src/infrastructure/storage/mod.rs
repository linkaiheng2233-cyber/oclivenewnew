//! # Role pack disk loading (including blueprint v2)
//!
//! ## Blueprint → kernel executable state (data flow)
//!
//! ```text
//! roles/{id}/pipeline.ocblueprint
//!   → read file + JSON parse (oclive_validation::load_blueprint_v2_for_role_dir)
//!   → schema / business validation (slot_registry, groups, interaction_mode…)
//!   → Role { slot_registry, plugin_backends, slot_groups, … }
//!   → PluginHost::resolve_for_role / resolve_for_effective_backends
//!   → SlotResolver::resolve → ResolvedRoleSlots
//!   → process_message / co_present → SlotRunner merge execution by slot type
//! ```
//!
//! - **`module_relations`**: not written to the blueprint file; frontend `buildBlueprintEdges(slot_registry)` **derives edges read-only**.
//! - **legacy**: when `pipeline.ocblueprint` is absent, fall back to the six-slot `manifest.json` + `settings.json` path.

use crate::models::{LlmBackend, Role};
use std::path::PathBuf;

/// Role pack storage manager.
///
/// Loads and saves role configuration from the filesystem.
#[derive(Debug, Clone)]
pub struct RoleStorage {
    pub(crate) roles_dir: PathBuf,
}

mod blueprint;
mod role;
mod scene;
mod user_identities;

/// Matches values injected by oclive-launcher: `ollama` / `remote` (case-insensitive).
#[must_use]
pub fn pick_llm_backend_env_override() -> Option<LlmBackend> {
    oclive_kernel_runtime::domain::plugin_resolution::pick_llm_backend_env_override()
}

/// Matches values injected by oclive-launcher: `ollama` / `remote` (case-insensitive); overrides on-disk `plugin_backends.llm`.
pub(super) fn apply_llm_backend_env_override(role: &mut Role) {
    if let Some(v) = pick_llm_backend_env_override() {
        std::sync::Arc::make_mut(&mut role.plugin_backends).llm = v;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::role::IdentityBinding;
    use crate::models::{
        EvolutionBounds, EvolutionConfig, MemoryConfig, PersonalityDefaults, UserRelation,
    };
    use std::collections::HashMap;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_role_storage_new() {
        let storage = RoleStorage::new("./roles");
        assert_eq!(storage.roles_dir, PathBuf::from("./roles"));
    }

    #[test]
    fn test_load_all_roles_empty_dir() {
        let temp_dir = TempDir::new().unwrap();
        let storage = RoleStorage::new(temp_dir.path());
        let roles = storage.load_all_roles().unwrap();
        assert_eq!(roles.len(), 0);
    }

    #[test]
    fn test_save_and_load_role_manifest() {
        let temp_dir = TempDir::new().unwrap();
        let storage = RoleStorage::new(temp_dir.path());

        let mut topic_weights: HashMap<String, HashMap<String, f64>> = HashMap::new();
        topic_weights.insert(
            "default".to_string(),
            [("日常".to_string(), 0.5)].into_iter().collect(),
        );

        let role = Role {
            id: "test_role".to_string(),
            name: "Test Role".to_string(),
            description: "A test role".to_string(),
            version: "1.0.0".to_string(),
            author: "Test Author".to_string(),
            core_personality: "Test personality".to_string(),
            default_personality: PersonalityDefaults {
                stubbornness: 0.5,
                clinginess: 0.5,
                sensitivity: 0.5,
                assertiveness: 0.5,
                forgiveness: 0.5,
                talkativeness: 0.5,
                warmth: 0.5,
            },
            evolution_bounds: EvolutionBounds::full_01(),
            user_relations: vec![UserRelation {
                id: "friend".into(),
                name: "好友".into(),
                prompt_hint: "".into(),
                favor_multiplier: 1.0,
                initial_favorability: 50.0,
            }],
            evolution_config: EvolutionConfig {
                event_impact_factor: 1.5,
                ai_analysis_interval: 20,
                max_change_per_event: 0.1,
                max_total_change: 0.6,
                personality_source: Default::default(),
            },
            memory_config: Some(MemoryConfig {
                scene_weight_multiplier: 2.0,
                topic_weights,
            }),
            default_relation: "friend".to_string(),
            ollama_model: Some("mumu:latest".to_string()),
            identity_binding: IdentityBinding::Global,
            life_trajectory: None,
            life_schedule: None,
            remote_presence: None,
            autonomous_scene: None,
            interaction_mode: None,
            min_runtime_version: None,
            dev_only: false,
            featured: false,
            deep_capsule_enabled: false,
            deep_capsule: None,
            preset_order: 999,
            plugin_backends: std::sync::Arc::new(crate::models::PluginBackends::default()),
            ui_config: crate::models::UiConfig::default(),
            knowledge_index: None,
            author_pack: None,
            reply_quality_anchor: None,
            time_config: crate::models::RoleTimeConfig::default(),
            pack_memory_config: crate::models::RolePackMemoryConfig::default(),
            pack_relation_config: crate::models::RolePackRelationConfig::default(),
            pack_evolution_config: crate::models::RolePackEvolutionConfig::default(),
            pack_chat_storage_config: crate::models::RolePackChatStorageConfig::default(),
            pack_portrait_catalog: Default::default(),
            portrait_catalog: None,
            pack_visual_presentation_config: Default::default(),
            pack_turn_thinking_config: None,
            pack_prompt_extra_sections: Vec::new(),
            slot_registry: None,
            slot_groups: None,
            runtime_config: None,
            pipeline_experimental: None,
            scene_ids: std::sync::Arc::from(Vec::<String>::new()),
            scene_config_cache: std::sync::Arc::new(parking_lot::RwLock::new(
                std::collections::HashMap::new(),
            )),
            scene_text_cache: std::sync::Arc::new(parking_lot::RwLock::new(
                std::collections::HashMap::new(),
            )),
            user_identity_catalog: None,
            pack_reply_post_processor_config: Default::default(),
            source_dir: None,
        };

        let role_dir = temp_dir.path().join("test_role");
        fs::create_dir_all(&role_dir).unwrap();
        let manifest_json = serde_json::to_string_pretty(
            &crate::models::role_manifest_disk::disk_manifest_from_role(&role),
        )
        .unwrap();
        fs::write(role_dir.join("manifest.json"), manifest_json).unwrap();
        let settings_json =
            serde_json::to_string_pretty(&crate::models::disk_role_settings_from_role(&role))
                .unwrap();
        fs::write(role_dir.join("settings.json"), settings_json).unwrap();

        let settings_path = role_dir.join("settings.json");
        assert!(settings_path.exists());

        let loaded_role = storage.load_role("test_role").unwrap();

        assert_eq!(loaded_role.id, "test_role");
        assert_eq!(loaded_role.name, "Test Role");
        assert_eq!(loaded_role.identity_binding, IdentityBinding::Global);
        assert_eq!(loaded_role.ollama_model.as_deref(), Some("mumu:latest"));
        assert_eq!(loaded_role.evolution_config.event_impact_factor, 1.5);
        assert_eq!(loaded_role.evolution_config.ai_analysis_interval, 20);
        assert_eq!(loaded_role.evolution_config.max_change_per_event, 0.1);
        assert_eq!(loaded_role.evolution_config.max_total_change, 0.6);
        let mem = loaded_role.memory_config.expect("memory_config");
        assert_eq!(mem.scene_weight_multiplier, 2.0);
        assert_eq!(
            mem.topic_weights.get("default").unwrap().get("日常"),
            Some(&0.5)
        );
    }

    #[test]
    fn plugin_state_ui_baseline_prefers_author_suggested_ui_when_non_empty() {
        let ui_from_disk = crate::models::UiConfig {
            shell: "from_ui_json".into(),
            ..Default::default()
        };
        let suggested = crate::models::UiConfig {
            shell: "from_author".into(),
            ..Default::default()
        };
        let role = Role {
            ui_config: ui_from_disk,
            author_pack: Some(crate::models::AuthorPackFile {
                suggested_ui: Some(suggested.clone()),
                ..Default::default()
            }),
            ..Role::default()
        };
        assert_eq!(role.plugin_state_ui_baseline().shell, "from_author");
    }

    #[test]
    fn plugin_state_ui_baseline_falls_back_to_ui_json_when_author_suggested_empty() {
        let ui_from_disk = crate::models::UiConfig {
            shell: "from_ui_json".into(),
            ..Default::default()
        };
        let role = Role {
            ui_config: ui_from_disk.clone(),
            author_pack: Some(crate::models::AuthorPackFile {
                suggested_ui: Some(crate::models::UiConfig::default()),
                ..Default::default()
            }),
            ..Role::default()
        };
        assert_eq!(role.plugin_state_ui_baseline().shell, "from_ui_json");
    }

    #[test]
    fn plugin_state_ui_baseline_without_author_uses_ui_json() {
        let ui_from_disk = crate::models::UiConfig {
            shell: "only_pack".into(),
            ..Default::default()
        };
        let role = Role {
            ui_config: ui_from_disk.clone(),
            author_pack: None,
            ..Role::default()
        };
        assert_eq!(role.plugin_state_ui_baseline().shell, "only_pack");
    }
}

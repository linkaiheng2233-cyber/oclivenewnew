//! Role management module.
//!
//! **Non-production conversation orchestration**: real requests are wired by
//! [`crate::domain::chat_engine::process_message`] through Repository / policy / LLM.
//! This module provides a synchronous, DB-free mini pipeline only for **unit tests**
//! and local algorithm demos, avoiding behavioral drift as the main orchestration
//! evolves in parallel — to assert production behavior, test `chat_engine` /
//! `chat_turn` or integration tests instead.

use crate::domain::memory_retrieval::MemoryRetrievalInput;
use crate::domain::plugin_host::ResolvedRolePlugins;
use crate::domain::{
    prompt_builder::effective_reply_quality_anchor, EventDetector, MemoryEngine, PersonalityEngine,
    PromptInput,
};
use crate::models::{
    Emotion, Event, EventType, Memory, PersonalitySource, PersonalityVector, Role,
};
use std::sync::Arc;

/// Role manager.
pub struct RoleManager {
    role: Role,
    personality: PersonalityVector,
    memory_engine: MemoryEngine,
    /// Plugin facade aligned with the main conversation pipeline (emotion / memory ranking / prompt).
    plugins: ResolvedRolePlugins,
}

impl RoleManager {
    /// Creates a new role manager.
    ///
    /// # Arguments
    /// * `role` - Role metadata
    /// * `personality` - Initial personality vector
    /// * `plugins` - Resolved plugin facade (same shape as main `process_message` pipeline)
    #[must_use]
    pub fn new(role: Role, personality: PersonalityVector, plugins: ResolvedRolePlugins) -> Self {
        Self {
            role,
            personality,
            memory_engine: MemoryEngine::new(),
            plugins,
        }
    }

    /// Sets the memory retrieval backend (for tests or demo paths aligned with `Role.plugin_backends.memory`).
    pub fn with_memory_retrieval(
        role: Role,
        personality: PersonalityVector,
        mut plugins: ResolvedRolePlugins,
        memory: Arc<dyn crate::domain::memory_retrieval::MemoryRetrieval>,
    ) -> Self {
        plugins.memory = memory;
        Self::new(role, personality, plugins)
    }

    /// Processes user input and produces a reply.
    ///
    /// # Arguments
    /// * `user_input` - User input text
    /// * `long_term_memories` - Long-term memory list
    ///
    /// # Returns
    /// (reply text, updated personality, detected event if any)
    ///
    /// # Panics
    ///
    /// Panics when `memory.rank_memories` fails on test/builtin paths (matches historical `expect` behavior).
    pub fn process_input(
        &mut self,
        user_input: &str,
        long_term_memories: &[Memory],
    ) -> (String, PersonalityVector, Option<Event>) {
        // 1. Analyze user emotion (aligned with `UserEmotionAnalyzer` / main conversation)
        let emotion_result = self.plugins.emotion.analyze(user_input).unwrap_or(
            crate::domain::emotion_analyzer::EmotionResult {
                joy: 0.0,
                sadness: 0.0,
                anger: 0.0,
                fear: 0.0,
                surprise: 0.0,
                disgust: 0.0,
                neutral: 1.0,
                extension: None,
            },
        );

        let user_emotion = emotion_result.to_emotion();
        let user_emotion_str = user_emotion.to_string();
        let user_emotion_prompt =
            crate::domain::emotion_analyzer::EmotionAnalyzer::format_for_prompt(&emotion_result);

        // 2. Detect events
        let event = EventDetector::detect(user_input, &user_emotion, &Emotion::Neutral).ok();

        // 3. Adjust personality (profile-first mode derives seven dims from profile; no direct vector push here)
        let mut updated_personality = self.personality.clone();
        if self.role.evolution_config.personality_source != PersonalitySource::Profile {
            updated_personality = PersonalityEngine::adjust_by_user_emotion(
                updated_personality,
                &user_emotion_str,
                &self.role.evolution_bounds,
            );

            if let Some(ref evt) = event {
                let impact = EventDetector::get_impact_factor(&evt.event_type);
                updated_personality = PersonalityEngine::evolve_by_event(
                    updated_personality,
                    impact,
                    &self.role.evolution_bounds,
                );
            }
        }

        // 4. Add short-term memory
        let memory = Memory {
            id: format!("mem_{}", chrono::Utc::now().timestamp()),
            role_id: self.role.id.clone(),
            content: user_input.to_string(),
            importance: 0.5,
            weight: 1.0,
            created_at: chrono::Utc::now(),
            scene_id: None,
            mention_count: 1,
            accessed_at: None,
        };
        self.memory_engine.add_short_term(memory);

        // 5. Fetch relevant memories (via MemoryRetrieval, aligned with main conversation pipeline)
        let relevant_memories = self
            .plugins
            .memory
            .rank_memories(MemoryRetrievalInput {
                memories: long_term_memories,
                user_query: user_input,
                scene_id: None,
                limit: 3,
            })
            .expect("rank_memories");

        // 6. Build prompt
        let prompt = self
            .plugins
            .prompt
            .build_prompt(&PromptInput {
                role: &self.role,
                personality: &updated_personality,
                memories: &relevant_memories,
                user_input,
                user_emotion: user_emotion_prompt.as_str(),
                user_relation_id: "",
                relation_hint: "",
                relation_before: "Stranger",
                favorability_before: 0.0,
                relation_preview: "Stranger",
                favorability_preview: 0.0,
                event_type: event
                    .as_ref()
                    .map(|e| &e.event_type)
                    .unwrap_or(&EventType::Ignore),
                impact_factor: event
                    .as_ref()
                    .map(|e| EventDetector::get_impact_factor(&e.event_type))
                    .unwrap_or(0.0),
                scene_label: "",
                scene_detail: "",
                topic_hint_line: "",
                life_context_line: "",
                worldview_snippet: "",
                mutable_personality: "",
                ephemeral_personality: "",
                reply_quality_anchor: effective_reply_quality_anchor(&self.role),
                previous_complex_emotion_narrative_hint: "",
                user_identity_template: "",
                user_identity_id: "",
                host_prompt_overlay: "",
                host_state_expression_hint: "",
                relation_transition_hint: "",
                extra_sections: &[],
                persona_override: None,
                previous_assistant_reply: "",
            })
            .expect("build_prompt");

        // 7. Update personality
        self.personality = updated_personality.clone();

        // Return prompt as reply (production path should use LLM generation)
        (prompt, updated_personality, event)
    }

    /// Returns the current personality.
    #[must_use]
    pub fn get_personality(&self) -> &PersonalityVector {
        &self.personality
    }

    /// Returns role metadata.
    #[must_use]
    pub fn get_role(&self) -> &Role {
        &self.role
    }

    /// Returns short-term memories.
    #[must_use]
    pub fn get_short_term_memories(&self) -> Vec<Memory> {
        self.memory_engine.get_short_term()
    }

    /// Clears short-term memories.
    pub fn clear_short_term_memories(&mut self) {
        self.memory_engine.clear_short_term();
    }

    /// Returns a personality summary string.
    #[must_use]
    pub fn get_personality_summary(&self) -> String {
        let traits = PersonalityEngine::get_dominant_traits(&self.personality);
        let stability = PersonalityEngine::calculate_stability_index(&self.personality);
        let extroversion = PersonalityEngine::calculate_extroversion_index(&self.personality);

        format!(
            "性格特征: {}\n稳定性: {:.1}%\n外向性: {:.1}%",
            traits.join(", "),
            stability * 100.0,
            extroversion * 100.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::EvolutionBounds;

    fn test_plugins(role: &Role) -> ResolvedRolePlugins {
        crate::infrastructure::plugin_wiring::test_plugin_host().resolve_for_role(role)
    }

    fn create_test_role() -> Role {
        Role {
            id: "test".to_string(),
            name: "TestBot".to_string(),
            description: "A test bot".to_string(),
            version: "1.0".to_string(),
            author: "Test".to_string(),
            core_personality: "Friendly".to_string(),
            default_personality: crate::models::PersonalityDefaults {
                stubbornness: 0.5,
                clinginess: 0.5,
                sensitivity: 0.5,
                assertiveness: 0.5,
                forgiveness: 0.5,
                talkativeness: 0.5,
                warmth: 0.5,
            },
            evolution_bounds: EvolutionBounds::full_01(),
            user_relations: vec![],
            evolution_config: crate::models::EvolutionConfig::default(),
            memory_config: None,
            default_relation: "friend".to_string(),
            ollama_model: None,
            identity_binding: crate::models::role::IdentityBinding::default(),
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
            slot_registry: None,
            slot_groups: None,
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
        }
    }

    fn create_test_personality() -> PersonalityVector {
        PersonalityVector {
            stubbornness: 0.4,
            clinginess: 0.5,
            sensitivity: 0.6,
            assertiveness: 0.5,
            forgiveness: 0.6,
            talkativeness: 0.6,
            warmth: 0.7,
        }
    }

    #[test]
    fn test_role_manager_creation() {
        let role = create_test_role();
        let personality = create_test_personality();
        let manager = RoleManager::new(role.clone(), personality.clone(), test_plugins(&role));

        assert_eq!(manager.get_role().id, "test");
        assert_eq!(manager.get_personality().warmth, 0.7);
    }

    #[test]
    fn test_process_input() {
        let role = create_test_role();
        let personality = create_test_personality();
        let mut manager = RoleManager::new(role.clone(), personality, test_plugins(&role));

        let (prompt, updated_personality, _event) = manager.process_input("你很棒", &[]);

        assert!(!prompt.is_empty());
        assert!(prompt.contains("TestBot"));
        assert!(updated_personality.warmth >= 0.0);
    }

    #[test]
    fn test_short_term_memory() {
        let role = create_test_role();
        let personality = create_test_personality();
        let mut manager = RoleManager::new(role.clone(), personality, test_plugins(&role));

        manager.process_input("Hello", &[]);
        let memories = manager.get_short_term_memories();

        assert_eq!(memories.len(), 1);
        assert_eq!(memories[0].content, "Hello");
    }

    #[test]
    fn test_clear_short_term_memories() {
        let role = create_test_role();
        let personality = create_test_personality();
        let mut manager = RoleManager::new(role.clone(), personality, test_plugins(&role));

        manager.process_input("Hello", &[]);
        manager.clear_short_term_memories();

        assert_eq!(manager.get_short_term_memories().len(), 0);
    }

    #[test]
    fn test_get_personality_summary() {
        let role = create_test_role();
        let personality = create_test_personality();
        let manager = RoleManager::new(role.clone(), personality, test_plugins(&role));

        let summary = manager.get_personality_summary();
        assert!(summary.contains("性格特征"));
        assert!(summary.contains("稳定性"));
        assert!(summary.contains("外向性"));
    }
}

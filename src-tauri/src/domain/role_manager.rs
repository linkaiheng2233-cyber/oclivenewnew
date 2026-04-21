//! 角色管理模块
//!
//! **非生产对话编排**：真实请求由 [`crate::domain::chat_engine::process_message`] 串联 Repository / 策略 / LLM。
//! 本模块仅在**单元测试**与本地算法演示中提供同步、无 DB 的迷你管线，避免与主编排并行演进时产生行为漂移——若需断言线上行为，应测 `chat_engine` / `chat_turn` 或集成测试。

use crate::domain::memory_retrieval::MemoryRetrievalInput;
use crate::domain::plugin_host::{PluginHost, ResolvedRolePlugins};
use crate::domain::{
    prompt_builder::effective_reply_quality_anchor, EventDetector, MemoryEngine, PersonalityEngine,
    PromptInput,
};
use crate::infrastructure::llm::{LlmClient, MockLlmClient};
use crate::models::{
    Emotion, Event, EventType, Memory, PersonalitySource, PersonalityVector, Role,
};
use std::sync::Arc;

fn resolved_plugins_dummy(role: &Role) -> ResolvedRolePlugins {
    let dummy_llm: Arc<dyn LlmClient> = Arc::new(MockLlmClient {
        reply: String::new(),
    });
    PluginHost::new(dummy_llm, None, std::env::temp_dir()).resolve_for_role(role)
}

/// 角色管理器
pub struct RoleManager {
    role: Role,
    personality: PersonalityVector,
    memory_engine: MemoryEngine,
    /// 与主对话管线一致的插件门面（情绪 / 记忆排序 / Prompt）
    plugins: ResolvedRolePlugins,
}

impl RoleManager {
    /// 创建新的角色管理器
    ///
    /// # Arguments
    /// * `role` - 角色信息
    /// * `personality` - 初始性格向量
    pub fn new(role: Role, personality: PersonalityVector) -> Self {
        let plugins = resolved_plugins_dummy(&role);
        Self {
            role,
            personality,
            memory_engine: MemoryEngine::new(),
            plugins,
        }
    }

    /// 指定记忆检索后端（用于测试或与 `Role.plugin_backends.memory` 对齐的演示路径）
    pub fn with_memory_retrieval(
        role: Role,
        personality: PersonalityVector,
        memory: Arc<dyn crate::domain::memory_retrieval::MemoryRetrieval>,
    ) -> Self {
        let mut plugins = resolved_plugins_dummy(&role);
        plugins.memory = memory;
        Self {
            role,
            personality,
            memory_engine: MemoryEngine::new(),
            plugins,
        }
    }

    /// 处理用户输入并生成回复
    ///
    /// # Arguments
    /// * `user_input` - 用户输入文本
    /// * `long_term_memories` - 长期记忆列表
    ///
    /// # Returns
    /// (回复文本, 更新后的性格, 检测到的事件)
    pub fn process_input(
        &mut self,
        user_input: &str,
        long_term_memories: &[Memory],
    ) -> (String, PersonalityVector, Option<Event>) {
        // 1. 分析用户情绪（与 `UserEmotionAnalyzer` / 主对话一致）
        let emotion_result = self.plugins.emotion.analyze(user_input).unwrap_or(
            crate::domain::emotion_analyzer::EmotionResult {
                joy: 0.0,
                sadness: 0.0,
                anger: 0.0,
                fear: 0.0,
                surprise: 0.0,
                disgust: 0.0,
                neutral: 1.0,
            },
        );

        let user_emotion = emotion_result.to_emotion();
        let user_emotion_str = user_emotion.to_string();
        let user_emotion_prompt =
            crate::domain::emotion_analyzer::EmotionAnalyzer::format_for_prompt(&emotion_result);

        // 2. 检测事件
        let event = EventDetector::detect(user_input, &user_emotion, &Emotion::Neutral).ok();

        // 3. 调整性格（人设优先模式由档案归纳七维，此处不直接推向量）
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

        // 4. 添加短期记忆
        let memory = Memory {
            id: format!("mem_{}", chrono::Utc::now().timestamp()),
            role_id: self.role.id.clone(),
            content: user_input.to_string(),
            importance: 0.5,
            weight: 1.0,
            created_at: chrono::Utc::now(),
            scene_id: None,
        };
        self.memory_engine.add_short_term(memory);

        // 5. 获取相关记忆（走 MemoryRetrieval，与主对话管线一致）
        let relevant_memories = self.plugins.memory.rank_memories(MemoryRetrievalInput {
            memories: long_term_memories,
            user_query: user_input,
            scene_id: None,
            limit: 3,
        });

        // 6. 构建提示词
        let prompt = self.plugins.prompt.build_prompt(&PromptInput {
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
            reply_quality_anchor: effective_reply_quality_anchor(&self.role),
        });

        // 7. 更新性格
        self.personality = updated_personality.clone();

        // 返回提示词作为回复（实际应由LLM生成）
        (prompt, updated_personality, event)
    }

    /// 获取当前性格
    pub fn get_personality(&self) -> &PersonalityVector {
        &self.personality
    }

    /// 获取角色信息
    pub fn get_role(&self) -> &Role {
        &self.role
    }

    /// 获取短期记忆
    pub fn get_short_term_memories(&self) -> Vec<Memory> {
        self.memory_engine.get_short_term()
    }

    /// 清空短期记忆
    pub fn clear_short_term_memories(&mut self) {
        self.memory_engine.clear_short_term();
    }

    /// 获取性格摘要
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
            plugin_backends: crate::models::PluginBackends::default(),
            ui_config: crate::models::UiConfig::default(),
            knowledge_index: None,
            author_pack: None,
            reply_quality_anchor: None,
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
        let manager = RoleManager::new(role.clone(), personality.clone());

        assert_eq!(manager.get_role().id, "test");
        assert_eq!(manager.get_personality().warmth, 0.7);
    }

    #[test]
    fn test_process_input() {
        let role = create_test_role();
        let personality = create_test_personality();
        let mut manager = RoleManager::new(role, personality);

        let (prompt, updated_personality, _event) = manager.process_input("你很棒", &[]);

        assert!(!prompt.is_empty());
        assert!(prompt.contains("TestBot"));
        assert!(updated_personality.warmth >= 0.0);
    }

    #[test]
    fn test_short_term_memory() {
        let role = create_test_role();
        let personality = create_test_personality();
        let mut manager = RoleManager::new(role, personality);

        manager.process_input("Hello", &[]);
        let memories = manager.get_short_term_memories();

        assert_eq!(memories.len(), 1);
        assert_eq!(memories[0].content, "Hello");
    }

    #[test]
    fn test_clear_short_term_memories() {
        let role = create_test_role();
        let personality = create_test_personality();
        let mut manager = RoleManager::new(role, personality);

        manager.process_input("Hello", &[]);
        manager.clear_short_term_memories();

        assert_eq!(manager.get_short_term_memories().len(), 0);
    }

    #[test]
    fn test_get_personality_summary() {
        let role = create_test_role();
        let personality = create_test_personality();
        let manager = RoleManager::new(role, personality);

        let summary = manager.get_personality_summary();
        assert!(summary.contains("性格特征"));
        assert!(summary.contains("稳定性"));
        assert!(summary.contains("外向性"));
    }
}

//! Infrastructure implementations of turn persistence ports.

use crate::domain::ports::conversation_persist::{
    ConversationPersistPort, TurnAppendResult, TurnAutoCleanupConfig, TurnPersistRequest,
};
use crate::domain::ports::turn_persistence::{ChatTurnAtomicInput, ChatTurnPersistencePort};
use crate::domain::ports::turn_policies::{TurnPolicies, TurnPoliciesPort};
use crate::infrastructure::chat_storage::{AutoCleanupConfig, ConversationStore};
use crate::infrastructure::db::{ChatTurnTxInput, DbManager};
use crate::infrastructure::policy_registry::PolicySet;
use crate::state::AppState;
use async_trait::async_trait;
use std::sync::Arc;

pub struct DbChatTurnPersistencePort {
    db: Arc<DbManager>,
}

impl DbChatTurnPersistencePort {
    #[must_use]
    pub fn new(db: Arc<DbManager>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ChatTurnPersistencePort for DbChatTurnPersistencePort {
    async fn apply_chat_turn_atomic(
        &self,
        input: ChatTurnAtomicInput<'_>,
    ) -> crate::error::Result<f64> {
        self.db
            .apply_chat_turn_atomic(ChatTurnTxInput {
                role_id: input.role_id,
                personality: input.personality,
                current_emotion: input.current_emotion,
                relation_state: input.relation_state,
                user_relation_key: input.user_relation_key,
                favor_delta: input.favor_delta,
                memory_content: input.memory_content,
                memory_scope: input.memory_scope,
                memory_importance: input.memory_importance,
                memory_fifo_limit: input.memory_fifo_limit,
                memory_similarity_threshold: input.memory_similarity_threshold,
                event: input.event,
                user_message: input.user_message,
                bot_reply: input.bot_reply,
                scene_id: input.scene_id,
            })
            .await
    }
}

pub struct AppTurnPoliciesPort<'a> {
    state: &'a AppState,
}

impl<'a> AppTurnPoliciesPort<'a> {
    #[must_use]
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }
}

impl TurnPoliciesPort for AppTurnPoliciesPort<'_> {
    fn policies_for_scene(&self, scene_id: Option<&str>) -> TurnPolicies {
        let set: Arc<PolicySet> = self.state.policies_for_scene(scene_id);
        TurnPolicies {
            memory_fifo_limit: set.memory.fifo_limit(),
        }
    }
}

pub struct StoreConversationPersistPort {
    store: Arc<dyn ConversationStore>,
}

impl StoreConversationPersistPort {
    #[must_use]
    pub fn new(store: Arc<dyn ConversationStore>) -> Self {
        Self { store }
    }
}

fn to_infra_cleanup(cfg: &TurnAutoCleanupConfig) -> AutoCleanupConfig {
    AutoCleanupConfig {
        auto_cleanup_days: cfg.auto_cleanup_days,
        auto_cleanup_max_sessions: cfg.auto_cleanup_max_sessions,
        chat_storage_location: cfg.chat_storage_location.clone(),
    }
}

#[async_trait]
impl ConversationPersistPort for StoreConversationPersistPort {
    async fn append_turn(
        &self,
        input: TurnPersistRequest,
    ) -> crate::error::Result<TurnAppendResult> {
        let stored = self
            .store
            .append_turn(crate::infrastructure::chat_storage::TurnPersistInput {
                idempotency_key: input.idempotency_key,
                session_id: input.session_id,
                role_id: input.role_id,
                scene_id: input.scene_id,
                user_message: input.user_message,
                user_message_hidden: input.user_message_hidden,
                assistant_reply: input.assistant_reply,
                reply_is_fallback: input.reply_is_fallback,
                model_name: input.model_name,
                response_ms: input.response_ms,
                user_emotion: input.user_emotion,
                bot_emotion: input.bot_emotion,
                bot_emotion_source: input.bot_emotion_source,
                bot_emotion_labels: input.bot_emotion_labels,
                user_emotion_scores: input.user_emotion_scores,
                emotion_pattern: input.emotion_pattern,
                emotion_confidence: input.emotion_confidence,
                emotion_intensity: input.emotion_intensity,
                emotion_dissonance: input.emotion_dissonance,
                emotion_hint: input.emotion_hint,
                reply_segments: input.reply_segments,
                reply_segment_delays_ms: input.reply_segment_delays_ms,
                max_messages_per_session: input.max_messages_per_session,
                auto_cleanup_config: to_infra_cleanup(&input.auto_cleanup_config),
                chat_storage_location: input.chat_storage_location,
            })
            .await?;
        Ok(TurnAppendResult {
            user_message_id: stored.user_message_id,
            assistant_message_id: stored.assistant_message_id,
            user_message_timestamp: stored.user_message_timestamp,
            assistant_message_timestamp: stored.assistant_message_timestamp,
        })
    }
}

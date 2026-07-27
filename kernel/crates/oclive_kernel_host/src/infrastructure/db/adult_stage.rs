//! Durable staging area for background-generated adult interaction beats.
//!
//! A staged beat is structured text only. It becomes part of short-term memory
//! and chat history only when the foreground explicitly commits it.

use super::DbManager;
use crate::error::{AppError, Result};
use crate::models::dto::{AdultStagedBeatDto, SendMessageResponse};
use chrono::Utc;
use sqlx::Row;
use std::sync::atomic::Ordering;
use uuid::Uuid;

#[derive(Debug)]
pub struct StoredAdultStageBeat {
    pub generation_id: String,
    pub sequence: u32,
    pub status: String,
    pub response: SendMessageResponse,
    pub transcript_reply: String,
    pub model_name: Option<String>,
    pub response_ms: u64,
    pub bot_emotion: Option<String>,
}

fn stage_error(message: impl Into<String>) -> AppError {
    AppError::InvalidParameter(format!("adult stage: {}", message.into()))
}

impl DbManager {
    pub async fn begin_adult_stage_generation(
        &self,
        session_id: &str,
        role_id: &str,
        scene_id: &str,
    ) -> Result<String> {
        let now = Utc::now().to_rfc3339();
        let generation_id = Uuid::new_v4().to_string();
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        sqlx::query(
            "DELETE FROM adult_staged_beats
             WHERE generation_id IN (
               SELECT generation_id FROM adult_stage_generations
               WHERE status != 'active' AND datetime(updated_at) < datetime('now', '-7 days')
             )",
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        sqlx::query(
            "DELETE FROM adult_stage_generations
             WHERE status != 'active' AND datetime(updated_at) < datetime('now', '-7 days')",
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        sqlx::query(
            "UPDATE adult_stage_generations
             SET status = 'cancelled', updated_at = ?
             WHERE session_id = ? AND scene_id = ? AND status = 'active'",
        )
        .bind(&now)
        .bind(session_id)
        .bind(scene_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        sqlx::query(
            "DELETE FROM adult_staged_beats
             WHERE status = 'pending'
               AND generation_id IN (
                 SELECT generation_id FROM adult_stage_generations
                 WHERE session_id = ? AND scene_id = ? AND status = 'cancelled'
               )",
        )
        .bind(session_id)
        .bind(scene_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        sqlx::query(
            "INSERT INTO adult_stage_generations
             (generation_id, session_id, role_id, scene_id, status, next_sequence,
              next_commit_sequence, created_at, updated_at)
             VALUES (?, ?, ?, ?, 'active', 0, 0, ?, ?)",
        )
        .bind(&generation_id)
        .bind(session_id)
        .bind(role_id)
        .bind(scene_id)
        .bind(&now)
        .bind(&now)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        tx.commit()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(generation_id)
    }

    pub async fn adult_stage_generation_active(
        &self,
        generation_id: &str,
        session_id: &str,
        role_id: &str,
        scene_id: &str,
    ) -> Result<bool> {
        let active: Option<i64> = sqlx::query_scalar(
            "SELECT 1 FROM adult_stage_generations
             WHERE generation_id = ? AND session_id = ? AND role_id = ?
               AND scene_id = ? AND status = 'active'",
        )
        .bind(generation_id)
        .bind(session_id)
        .bind(role_id)
        .bind(scene_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(active.is_some())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn store_adult_staged_beat(
        &self,
        generation_id: &str,
        session_id: &str,
        role_id: &str,
        scene_id: &str,
        sequence: u32,
        response: &SendMessageResponse,
        transcript_reply: &str,
        model_name: Option<&str>,
        response_ms: u64,
    ) -> Result<()> {
        let response_json = serde_json::to_string(response)?;
        let now = Utc::now().to_rfc3339();
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let inserted = sqlx::query(
            "INSERT INTO adult_staged_beats
             (generation_id, sequence, status, response_json, transcript_reply,
              model_name, response_ms, bot_emotion, created_at)
             SELECT generation_id, ?, 'pending', ?, ?, ?, ?, ?, ?
             FROM adult_stage_generations
             WHERE generation_id = ? AND session_id = ? AND role_id = ?
               AND scene_id = ? AND status = 'active' AND next_sequence = ?",
        )
        .bind(i64::from(sequence))
        .bind(response_json)
        .bind(transcript_reply)
        .bind(model_name)
        .bind(i64::try_from(response_ms).unwrap_or(i64::MAX))
        .bind(&response.bot_emotion)
        .bind(&now)
        .bind(generation_id)
        .bind(session_id)
        .bind(role_id)
        .bind(scene_id)
        .bind(i64::from(sequence))
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        if inserted.rows_affected() != 1 {
            tx.rollback().await.ok();
            return Err(stage_error(
                "generation was cancelled or sequence is no longer current",
            ));
        }
        sqlx::query(
            "UPDATE adult_stage_generations
             SET next_sequence = ?, updated_at = ?
             WHERE generation_id = ?",
        )
        .bind(i64::from(sequence) + 1)
        .bind(&now)
        .bind(generation_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        tx.commit()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn load_adult_staged_beat(
        &self,
        generation_id: &str,
        sequence: u32,
    ) -> Result<Option<StoredAdultStageBeat>> {
        let row = sqlx::query(
            "SELECT generation_id, sequence, status, response_json, transcript_reply,
                    model_name, response_ms, bot_emotion
             FROM adult_staged_beats
             WHERE generation_id = ? AND sequence = ?",
        )
        .bind(generation_id)
        .bind(i64::from(sequence))
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        row.map(|row| {
            let response_json: String = row.get("response_json");
            Ok(StoredAdultStageBeat {
                generation_id: row.get("generation_id"),
                sequence: u32::try_from(row.get::<i64, _>("sequence")).unwrap_or_default(),
                status: row.get("status"),
                response: serde_json::from_str(&response_json)?,
                transcript_reply: row.get("transcript_reply"),
                model_name: row.get("model_name"),
                response_ms: u64::try_from(row.get::<i64, _>("response_ms")).unwrap_or_default(),
                bot_emotion: row.get("bot_emotion"),
            })
        })
        .transpose()
    }

    pub async fn list_adult_staged_beats(
        &self,
        generation_id: &str,
    ) -> Result<Vec<AdultStagedBeatDto>> {
        let rows = sqlx::query(
            "SELECT sequence, response_json FROM adult_staged_beats
             WHERE generation_id = ? AND status IN ('pending', 'memory_committed')
             ORDER BY sequence ASC",
        )
        .bind(generation_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        rows.into_iter()
            .map(|row| {
                let response_json: String = row.get("response_json");
                Ok(AdultStagedBeatDto {
                    generation_id: generation_id.to_string(),
                    sequence: u32::try_from(row.get::<i64, _>("sequence")).unwrap_or_default(),
                    response: serde_json::from_str(&response_json)?,
                })
            })
            .collect()
    }

    pub async fn adult_stage_generation_state(
        &self,
        generation_id: &str,
    ) -> Result<Option<(bool, u32)>> {
        let row = sqlx::query(
            "SELECT status, next_sequence FROM adult_stage_generations WHERE generation_id = ?",
        )
        .bind(generation_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(row.map(|row| {
            let status: String = row.get("status");
            let next = u32::try_from(row.get::<i64, _>("next_sequence")).unwrap_or_default();
            (status == "active", next)
        }))
    }

    pub async fn pending_adult_stage_transcripts_before(
        &self,
        generation_id: &str,
        sequence: u32,
    ) -> Result<Vec<String>> {
        sqlx::query_scalar(
            "SELECT transcript_reply FROM adult_staged_beats
             WHERE generation_id = ? AND sequence < ?
               AND status IN ('pending', 'memory_committed')
             ORDER BY sequence ASC",
        )
        .bind(generation_id)
        .bind(i64::from(sequence))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    /// Commit only the staged beat's short-term conversational context. Long
    /// term memory, favorability, events and personality remain tied to real
    /// user turns; synthetic background beats must not amplify them.
    pub async fn commit_adult_staged_short_term(
        &self,
        staged: &StoredAdultStageBeat,
        session_id: &str,
        scene_id: &str,
        hidden_user_message: &str,
        fifo_limit: i32,
    ) -> Result<()> {
        let generation_id = staged.generation_id.as_str();
        let sequence = staged.sequence;
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let row = sqlx::query(
            "SELECT b.status, g.next_commit_sequence
             FROM adult_staged_beats b
             JOIN adult_stage_generations g ON g.generation_id = b.generation_id
             WHERE b.generation_id = ? AND b.sequence = ? AND g.session_id = ? AND g.scene_id = ?
               AND g.status = 'active'",
        )
        .bind(generation_id)
        .bind(i64::from(sequence))
        .bind(session_id)
        .bind(scene_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| stage_error("beat not found for this chat"))?;
        let status: String = row.get("status");
        let next_commit =
            u32::try_from(row.get::<i64, _>("next_commit_sequence")).unwrap_or_default();
        if status == "committed" || status == "memory_committed" {
            tx.rollback().await.ok();
            return Ok(());
        }
        if status != "pending" || next_commit != sequence {
            tx.rollback().await.ok();
            return Err(stage_error("beats must be committed in sequence"));
        }
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO short_term_memory
             (role_id, user_input, bot_reply, emotion, scene, created_at, content_scope)
             VALUES (?, ?, ?, ?, ?, ?, 'adult')",
        )
        .bind(session_id)
        .bind(hidden_user_message)
        .bind(staged.transcript_reply.as_str())
        .bind(staged.bot_emotion.as_deref().unwrap_or("neutral"))
        .bind(scene_id)
        .bind(&now)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let limit = i64::from(fifo_limit.max(1));
        sqlx::query(
            "DELETE FROM short_term_memory
             WHERE role_id = ? AND id NOT IN (
               SELECT id FROM short_term_memory WHERE role_id = ?
               ORDER BY id DESC LIMIT ?
             )",
        )
        .bind(session_id)
        .bind(session_id)
        .bind(limit)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let final_short_term_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM short_term_memory WHERE role_id = ?")
                .bind(session_id)
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        sqlx::query(
            "UPDATE adult_staged_beats SET status = 'memory_committed' WHERE generation_id = ? AND sequence = ?",
        )
        .bind(generation_id)
        .bind(i64::from(sequence))
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        sqlx::query(
            "UPDATE adult_stage_generations SET next_commit_sequence = ?, updated_at = ?
             WHERE generation_id = ?",
        )
        .bind(i64::from(sequence) + 1)
        .bind(&now)
        .bind(generation_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        tx.commit()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        self.short_term_row_counts
            .entry(session_id.to_string())
            .or_insert_with(|| std::sync::atomic::AtomicI64::new(0))
            .store(final_short_term_count, Ordering::Relaxed);
        Ok(())
    }

    pub async fn finish_adult_staged_beat(
        &self,
        generation_id: &str,
        sequence: u32,
        interaction_ended: bool,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        sqlx::query(
            "UPDATE adult_staged_beats
             SET status = 'committed', committed_at = ?
             WHERE generation_id = ? AND sequence = ? AND status = 'memory_committed'",
        )
        .bind(&now)
        .bind(generation_id)
        .bind(i64::from(sequence))
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        if interaction_ended {
            sqlx::query(
                "UPDATE adult_stage_generations
                 SET status = 'completed', updated_at = ?
                 WHERE generation_id = ?",
            )
            .bind(&now)
            .bind(generation_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        }
        tx.commit()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn cancel_adult_stage_generation(
        &self,
        generation_id: &str,
        session_id: &str,
        role_id: &str,
        scene_id: &str,
    ) -> Result<bool> {
        let now = Utc::now().to_rfc3339();
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let changed = sqlx::query(
            "UPDATE adult_stage_generations SET status = 'cancelled', updated_at = ?
             WHERE generation_id = ? AND session_id = ? AND role_id = ? AND scene_id = ?
               AND status = 'active'",
        )
        .bind(&now)
        .bind(generation_id)
        .bind(session_id)
        .bind(role_id)
        .bind(scene_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .rows_affected()
            > 0;
        sqlx::query(
            "DELETE FROM adult_staged_beats WHERE generation_id = ? AND status = 'pending'",
        )
        .bind(generation_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        tx.commit()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(changed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::adult_stage::{
        begin_adult_stage_generation, commit_adult_staged_beat, generate_adult_staged_beat,
    };
    use crate::infrastructure::llm::MockLlmClient;
    use crate::models::dto::{
        AdultBeatDto, AdultInteractionAction, AdultInteractionRequest, AdultInteractionState,
        BeginAdultStageGenerationRequest, CommitAdultStagedBeatRequest, EmotionDto, PresenceMode,
        StageAdultBeatRequest, API_VERSION, SCHEMA_VERSION,
    };
    use crate::state::AppState;
    use std::sync::Arc;

    fn open_adult_request() -> AdultInteractionRequest {
        AdultInteractionRequest {
            confirmed_adult: true,
            global_enabled: true,
            role_enabled: true,
            interaction_active: true,
            action: AdultInteractionAction::Continue,
            stage: None,
        }
    }

    fn roles_dir() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../distros/chat-pro/roles")
    }

    fn response(state: AdultInteractionState) -> SendMessageResponse {
        SendMessageResponse {
            api_version: API_VERSION,
            schema: SCHEMA_VERSION,
            presence_mode: PresenceMode::CoPresent,
            display_metrics: None,
            relation_state: "Friend".to_string(),
            reply: "dialogue".to_string(),
            adult_beat: Some(AdultBeatDto {
                dialogue: "dialogue".to_string(),
                narration: "narration".to_string(),
                interaction_state: state,
                next_beat_interval_ms: Some(10),
            }),
            emotion: EmotionDto {
                joy: 0.0,
                sadness: 0.0,
                anger: 0.0,
                fear: 0.0,
                surprise: 0.0,
                disgust: 0.0,
                neutral: 1.0,
            },
            bot_emotion: "neutral".to_string(),
            portrait_emotion: "neutral".to_string(),
            visual_state_id: None,
            performance_directive: None,
            favorability_delta: 0.0,
            favorability_current: 0.0,
            events: vec![],
            scene_id: "home".to_string(),
            offer_destination_picker: false,
            offer_together_travel: false,
            reply_is_fallback: false,
            llm_fallback_reason: None,
            knowledge_chunks_in_prompt: 0,
            timestamp: 0,
            user_message_id: None,
            assistant_message_id: None,
            user_message_timestamp: None,
            assistant_message_timestamp: None,
            chat_persist_failed: None,
            chat_persist_error: None,
            dual_core_degraded: None,
            raw_reply: None,
            llm_prompt_eval_ms: None,
        }
    }

    #[tokio::test]
    async fn staged_beats_are_ordered_idempotent_and_cancellable() {
        let pool = crate::infrastructure::test_db::connect_memory_migrated().await;
        let db = DbManager::new(pool);
        db.ensure_role_runtime("role").await.expect("runtime");
        let generation = db
            .begin_adult_stage_generation("role", "role", "home")
            .await
            .expect("begin");
        db.store_adult_staged_beat(
            &generation,
            "role",
            "role",
            "home",
            0,
            &response(AdultInteractionState::Active),
            "turn 0",
            Some("model"),
            10,
        )
        .await
        .expect("stage 0");
        db.store_adult_staged_beat(
            &generation,
            "role",
            "role",
            "home",
            1,
            &response(AdultInteractionState::Active),
            "turn 1",
            Some("model"),
            10,
        )
        .await
        .expect("stage 1");
        let staged_zero = db
            .load_adult_staged_beat(&generation, 0)
            .await
            .expect("load 0")
            .expect("staged 0");
        let staged_one = db
            .load_adult_staged_beat(&generation, 1)
            .await
            .expect("load 1")
            .expect("staged 1");

        let out_of_order = db
            .commit_adult_staged_short_term(&staged_one, "role", "home", "continue", 500)
            .await;
        assert!(out_of_order.is_err());
        db.commit_adult_staged_short_term(&staged_zero, "role", "home", "continue", 500)
            .await
            .expect("commit 0");
        db.commit_adult_staged_short_term(&staged_zero, "role", "home", "continue", 500)
            .await
            .expect("retry 0");
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM short_term_memory WHERE role_id = 'role'")
                .fetch_one(&db.pool)
                .await
                .expect("count");
        assert_eq!(count, 1);

        db.cancel_adult_stage_generation(&generation, "role", "role", "home")
            .await
            .expect("cancel");
        assert!(!db
            .adult_stage_generation_active(&generation, "role", "role", "home")
            .await
            .expect("state"));
        assert!(db
            .load_adult_staged_beat(&generation, 1)
            .await
            .expect("load")
            .is_none());
    }

    #[tokio::test]
    async fn new_generation_invalidates_old_pending_beats() {
        let pool = crate::infrastructure::test_db::connect_memory_migrated().await;
        let db = DbManager::new(pool);
        let first = db
            .begin_adult_stage_generation("role", "role", "home")
            .await
            .expect("begin first");
        db.store_adult_staged_beat(
            &first,
            "role",
            "role",
            "home",
            0,
            &response(AdultInteractionState::Active),
            "turn",
            None,
            0,
        )
        .await
        .expect("stage");
        let second = db
            .begin_adult_stage_generation("role", "role", "home")
            .await
            .expect("begin second");
        assert_ne!(first, second);
        assert!(db
            .load_adult_staged_beat(&first, 0)
            .await
            .expect("load")
            .is_none());
        assert!(db
            .adult_stage_generation_active(&second, "role", "role", "home")
            .await
            .expect("active"));
    }

    #[tokio::test]
    async fn cancellation_race_never_leaves_an_unshown_pending_beat() {
        let pool = crate::infrastructure::test_db::connect_memory_migrated().await;
        let db = DbManager::new(pool);
        for sequence in 0..32 {
            let generation = db
                .begin_adult_stage_generation("role", "role", "home")
                .await
                .expect("begin");
            let transcript = format!("turn {sequence}");
            let staged_response = response(AdultInteractionState::Active);
            let store = db.store_adult_staged_beat(
                &generation,
                "role",
                "role",
                "home",
                0,
                &staged_response,
                transcript.as_str(),
                Some("model"),
                10,
            );
            let cancel = db.cancel_adult_stage_generation(&generation, "role", "role", "home");
            let (_store_result, cancel_result) = tokio::join!(store, cancel);
            cancel_result.expect("cancel");

            assert!(!db
                .adult_stage_generation_active(&generation, "role", "role", "home")
                .await
                .expect("inactive"));
            assert!(db
                .list_adult_staged_beats(&generation)
                .await
                .expect("list")
                .is_empty());
        }
    }

    #[tokio::test]
    async fn staged_generation_has_no_visible_side_effect_until_commit() {
        let llm = Arc::new(MockLlmClient {
            reply: r#"{"dialogue":"next","narration":"moves","interaction_state":"active","next_beat_interval_ms":10}"#
                .to_string(),
        });
        let state = AppState::new_in_memory_with_llm(llm, roles_dir())
            .await
            .expect("state");
        let begun = begin_adult_stage_generation(
            &state,
            BeginAdultStageGenerationRequest {
                role_id: "gentle-landlady".to_string(),
                scene_id: Some("default".to_string()),
                session_id: None,
                adult: open_adult_request(),
            },
        )
        .await
        .expect("begin");
        let staged = generate_adult_staged_beat(
            &state,
            StageAdultBeatRequest {
                role_id: "gentle-landlady".to_string(),
                scene_id: Some("default".to_string()),
                session_id: None,
                generation_id: begun.generation_id.clone(),
                sequence: 0,
                adult: open_adult_request(),
            },
        )
        .await
        .expect("generate");
        assert_eq!(
            staged
                .response
                .adult_beat
                .as_ref()
                .expect("structured")
                .interaction_state,
            AdultInteractionState::Active
        );
        assert!(state
            .conversation_store
            .fetch_messages("gentle-landlady", 10, 0)
            .await
            .expect("before messages")
            .is_empty());
        let short_before: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM short_term_memory WHERE role_id = 'gentle-landlady'",
        )
        .fetch_one(&state.db_manager.pool)
        .await
        .expect("short before");
        assert_eq!(short_before, 0);

        let committed = commit_adult_staged_beat(
            &state,
            CommitAdultStagedBeatRequest {
                role_id: "gentle-landlady".to_string(),
                scene_id: Some("default".to_string()),
                session_id: None,
                generation_id: begun.generation_id,
                sequence: 0,
            },
        )
        .await
        .expect("commit");
        assert!(committed.assistant_message_id.is_some());
        assert_eq!(
            state
                .conversation_store
                .fetch_messages("gentle-landlady", 10, 0)
                .await
                .expect("after messages")
                .len(),
            2
        );
        let short_after: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM short_term_memory WHERE role_id = 'gentle-landlady'",
        )
        .fetch_one(&state.db_manager.pool)
        .await
        .expect("short after");
        assert_eq!(short_after, 1);
    }
}

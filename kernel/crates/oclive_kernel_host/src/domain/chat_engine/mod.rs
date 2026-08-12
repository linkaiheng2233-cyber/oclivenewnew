//! Chat orchestration: wires domain modules with Repository / LLM.
//!
//! Pure turn logic lives in [`super::chat_turn`]; this module handles async orchestration and `AppState`.
//! Scene and favorability sub-logic: [`context`], [`scene`], [`favor`].

pub mod chat_stage;
pub(crate) mod context;
pub(crate) mod dispatch;
pub(crate) mod favor;
pub mod message_error;
pub(crate) mod minimal_response;
pub mod plugin_resolve;
mod presence;
mod process_message;
pub(crate) mod relation_snapshot;
mod scene;
pub(crate) mod staged;
pub mod turn_context;
pub(crate) mod turn_error;
pub(crate) mod turn_pipeline;
pub mod turn_prefetch;

pub use process_message::{process_message, process_message_stream};

use turn_context::TurnContext;
use turn_pipeline::{execute_turn, TurnMode};

use crate::domain::chat_engine::relation_snapshot::load_relation_snapshot;
use crate::domain::remote_life_prompt::compose_remote_stub_reply;
use crate::domain::user_identity::resolve_effective_user_relation_key;
use crate::error::Result;
use crate::models::dto::{
    DisplayMetricsDto, EmotionDto, PresenceMode, SendMessageResponse, API_VERSION, SCHEMA_VERSION,
};
use crate::models::{PluginBackends, PluginBackendsSourceMap, Role};
use crate::state::AppState;
use std::sync::Arc;

pub(super) fn emotion_to_dto(r: &crate::domain::emotion_analyzer::EmotionResult) -> EmotionDto {
    EmotionDto {
        joy: r.joy as f32,
        sadness: r.sadness as f32,
        anger: r.anger as f32,
        fear: r.fear as f32,
        surprise: r.surprise as f32,
        disgust: r.disgust as f32,
        neutral: r.neutral as f32,
    }
}

pub(super) fn backend_resolution_summary(
    effective: &PluginBackends,
    sources: &PluginBackendsSourceMap,
) -> String {
    format!(
        "mem={:?}({:?}) emotion={:?}({:?}) event={:?}({:?}) prompt={:?}({:?}) llm={:?}({:?}) agent={:?}({:?})",
        effective.memory,
        sources.memory,
        effective.emotion,
        sources.emotion,
        effective.event,
        sources.event,
        effective.prompt,
        sources.prompt,
        effective.llm,
        sources.llm,
        effective.agent,
        sources.agent
    )
}

/// Session-scoped SQLite namespace: HTTP trial chat with `session_id` is isolated from the default conversation without one.
pub fn conversation_state_role_id(manifest_role_id: &str, session_id: Option<&str>) -> String {
    /// Caps SQLite key and log length so abnormally long `session_id` values cannot blow storage.
    const MAX_SUFFIX_CHARS: usize = 64;
    const MAX_TOTAL_CHARS: usize = 256;

    let sid = session_id.map(str::trim).filter(|s| !s.is_empty());
    match sid {
        None => manifest_role_id.chars().take(MAX_TOTAL_CHARS).collect(),
        Some(s) => {
            let safe: String = s
                .chars()
                .map(|c| {
                    if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                        c
                    } else {
                        '_'
                    }
                })
                .take(MAX_SUFFIX_CHARS)
                .collect();
            let out = format!("{}__sess__{}", manifest_role_id, safe);
            out.chars().take(MAX_TOTAL_CHARS).collect()
        }
    }
}

/// Remote-presence + off: stub reply; **does not** write short-term memory / events / favorability (avoids favor gain without dialogue).
pub(super) async fn process_remote_stub(ctx: &TurnContext<'_>) -> Result<SendMessageResponse> {
    let state = ctx.state;
    let req = ctx.req;
    let role = ctx.role;
    let scene_id = ctx.scene_id;
    let t0 = ctx.t0;
    let srid = ctx.srid;
    let preflight_ms = ctx.preflight_ms;
    let role_id = req.role_id.as_str();
    let user_message = req.user_message.as_str();
    let pl = &ctx.pl;
    let emotion_result = pl.emotion.analyze(user_message)?;
    let user_relation_key: String =
        resolve_effective_user_relation_key(state, role, srid, Some(scene_id)).await?;
    let snapshot = load_relation_snapshot(state, srid, user_relation_key.as_str(), None).await?;
    let relation_before = snapshot.relation_state;
    let favorability_before = snapshot.favorability;
    let portrait_emotion_str = snapshot.portrait_emotion;
    let reply = compose_remote_stub_reply(role);
    let duration_ms = t0.elapsed().as_millis() as u64;
    tracing::info!(
        target: "oclive_chat",
        "send_message remote_stub role_id={} scene_id={} duration_ms={}",
        role_id,
        scene_id,
        duration_ms
    );
    tracing::debug!(
        target: "oclive_chat",
        "send_message remote_stub timing preflight_ms={} duration_ms={}",
        preflight_ms,
        duration_ms
    );
    Ok(SendMessageResponse {
        api_version: API_VERSION,
        schema: SCHEMA_VERSION,
        presence_mode: PresenceMode::RemoteStub,
        display_metrics: Some(DisplayMetricsDto {
            favor: favorability_before,
            relation_summary: relation_before.clone(),
            traits: vec![],
        }),
        relation_state: relation_before,
        reply,
        adult_beat: None,
        emotion: emotion_to_dto(&emotion_result),
        bot_emotion: "neutral".to_string(),
        portrait_emotion: portrait_emotion_str,
        visual_state_id: None,
        performance_directive: None,
        favorability_delta: 0.0,
        favorability_current: favorability_before as f32,
        events: vec![],
        scene_id: scene_id.to_string(),
        offer_destination_picker: false,
        offer_together_travel: false,
        reply_is_fallback: false,
        llm_fallback_reason: None,
        knowledge_chunks_in_prompt: 0,
        timestamp: chrono::Utc::now().timestamp_millis(),
        user_message_id: None,
        assistant_message_id: None,
        user_message_timestamp: None,
        assistant_message_timestamp: None,
        chat_persist_failed: None,
        chat_persist_error: None,
        dual_core_degraded: None,
        raw_reply: None,
        llm_prompt_eval_ms: None,
    })
}

/// Remote-presence + on: dedicated LLM; skips event-impact detection, uses `Ignore` + zero amplitude in favorability (still updates short-term memory, etc.).
pub(super) async fn process_remote_life(ctx: &TurnContext<'_>) -> Result<SendMessageResponse> {
    execute_turn(ctx, TurnMode::RemoteLife)
        .await
        .map_err(Into::into)
}

pub(super) async fn ensure_role_loaded(state: &AppState, role_id: &str) -> Result<Arc<Role>> {
    state.load_role_cached_async(role_id).await
}

#[cfg(test)]
mod tests {
    use crate::domain::chat_engine::{backend_resolution_summary, conversation_state_role_id};
    use crate::domain::chat_turn_rules::{
        avoid_fast_promote_score, smooth_favor_delta_for_short_streak, soft_append_guard,
    };
    use crate::error::{AppError, Result};
    use crate::models::dto::SendMessageRequest;
    use crate::models::role_manifest_disk::disk_manifest_from_role;
    use crate::models::{
        disk_role_settings_from_role, EmotionBackend, Event, EventBackend, EventType, LlmBackend,
        MemoryBackend, PluginBackendSource, PluginBackends, PluginBackendsSourceMap, PromptBackend,
        Role, UserRelation,
    };
    use crate::state::AppState;
    use async_trait::async_trait;
    use oclive_kernel_contracts::LlmClient;
    use std::sync::Arc;

    #[test]
    fn conversation_state_role_id_none_matches_manifest_id() {
        assert_eq!(conversation_state_role_id("role_a", None), "role_a");
    }

    #[test]
    fn conversation_state_role_id_distinct_sessions() {
        let a = conversation_state_role_id("role_a", Some("sess-1"));
        let b = conversation_state_role_id("role_a", Some("sess-2"));
        assert_ne!(a, b);
        assert!(a.contains("__sess__"));
    }

    #[test]
    fn conversation_state_role_id_caps_total_length() {
        let long = "x".repeat(400);
        let out = conversation_state_role_id("r", Some(&long));
        assert!(out.chars().count() <= 256);
    }

    #[test]
    fn backend_resolution_summary_contains_backend_and_source_pairs() {
        let effective = PluginBackends {
            memory: MemoryBackend::Remote,
            emotion: EmotionBackend::Builtin,
            event: EventBackend::Builtin,
            prompt: PromptBackend::Remote,
            llm: LlmBackend::Remote,
            ..Default::default()
        };
        let sources = PluginBackendsSourceMap {
            memory: PluginBackendSource::SessionOverride,
            emotion: PluginBackendSource::PackDefault,
            event: PluginBackendSource::SessionOverride,
            prompt: PluginBackendSource::PackDefault,
            llm: PluginBackendSource::EnvOverride,
            agent: PluginBackendSource::PackDefault,
        };
        let out = backend_resolution_summary(&effective, &sources);
        assert!(out.contains("mem=Remote(SessionOverride)"));
        assert!(out.contains("llm=Remote(EnvOverride)"));
    }

    #[test]
    fn soft_append_triggers_for_quarrel_with_sweet_words() {
        let reply = "宝贝别生气呀，抱抱你，我最想你了";
        let out = soft_append_guard(reply, &EventType::Quarrel, -0.3, "Friend");
        assert!(out.len() > reply.len());
        assert!(out.contains("先"));
    }

    #[test]
    fn soft_append_triggers_for_low_stage_with_strong_promise() {
        let reply = "我想和你永远在一起，这辈子都不离不弃";
        let out = soft_append_guard(reply, &EventType::Praise, 0.4, "Stranger");
        assert!(out.len() > reply.len());
        assert!(out.contains("慢慢"));
    }

    #[test]
    fn soft_append_not_triggered_for_normal_positive_reply() {
        let reply = "今天和你聊天很开心，我们继续聊聊最近的事吧。";
        let out = soft_append_guard(reply, &EventType::Praise, 0.5, "Friend");
        assert_eq!(out, reply);
    }

    #[test]
    fn avoid_fast_promote_detects_consecutive_positive_streak() {
        let recent = vec![
            Event {
                event_type: EventType::Praise,
                user_emotion: "joy".to_string(),
                bot_emotion: "joy".to_string(),
            },
            Event {
                event_type: EventType::Confession,
                user_emotion: "joy".to_string(),
                bot_emotion: "surprise".to_string(),
            },
        ];
        let score = avoid_fast_promote_score(&EventType::Praise, 0.7, &recent);
        assert!(score >= 0.7);
    }

    #[test]
    fn avoid_fast_promote_ignores_weak_or_broken_streak() {
        let weak = avoid_fast_promote_score(&EventType::Praise, 0.3, &[]);
        assert_eq!(weak, 0.0);

        let broken = vec![
            Event {
                event_type: EventType::Joke,
                user_emotion: "neutral".to_string(),
                bot_emotion: "joy".to_string(),
            },
            Event {
                event_type: EventType::Praise,
                user_emotion: "joy".to_string(),
                bot_emotion: "joy".to_string(),
            },
        ];
        let broken_score = avoid_fast_promote_score(&EventType::Confession, 0.8, &broken);
        assert_eq!(broken_score, 0.0);
    }

    #[test]
    fn avoid_fast_promote_does_not_apply_to_negative_events() {
        let recent = vec![Event {
            event_type: EventType::Praise,
            user_emotion: "joy".to_string(),
            bot_emotion: "joy".to_string(),
        }];
        let score = avoid_fast_promote_score(&EventType::Quarrel, -0.8, &recent);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn favor_delta_smoothing_applies_on_consecutive_positive_streak() {
        let recent = vec![
            Event {
                event_type: EventType::Praise,
                user_emotion: "joy".to_string(),
                bot_emotion: "joy".to_string(),
            },
            Event {
                event_type: EventType::Confession,
                user_emotion: "joy".to_string(),
                bot_emotion: "joy".to_string(),
            },
        ];
        let out = smooth_favor_delta_for_short_streak(0.1, &recent);
        assert!(out < 0.1);
        assert!(out > 0.08);
    }

    #[test]
    fn favor_delta_smoothing_keeps_non_streak_or_low_amplitude_nearly_same() {
        let broken = vec![
            Event {
                event_type: EventType::Quarrel,
                user_emotion: "anger".to_string(),
                bot_emotion: "anger".to_string(),
            },
            Event {
                event_type: EventType::Praise,
                user_emotion: "joy".to_string(),
                bot_emotion: "joy".to_string(),
            },
        ];
        let unchanged = smooth_favor_delta_for_short_streak(0.1, &broken);
        assert_eq!(unchanged, 0.1);

        let low_amp = smooth_favor_delta_for_short_streak(0.02, &broken);
        assert_eq!(low_amp, 0.02);
    }

    #[test]
    fn favor_delta_smoothing_supports_negative_streak() {
        let recent = vec![
            Event {
                event_type: EventType::Quarrel,
                user_emotion: "anger".to_string(),
                bot_emotion: "anger".to_string(),
            },
            Event {
                event_type: EventType::Complaint,
                user_emotion: "sadness".to_string(),
                bot_emotion: "sadness".to_string(),
            },
        ];
        let out = smooth_favor_delta_for_short_streak(-0.1, &recent);
        assert!(out > -0.1);
        assert!(out < -0.08);
    }

    /// LLM backend that always fails; drives the graceful-degradation path
    /// (fallback reply + previous-emotion keep) for the M1 error-injection case.
    struct FailingLlmClient;

    #[async_trait]
    impl LlmClient for FailingLlmClient {
        async fn generate(&self, _model: &str, _prompt: &str) -> Result<String> {
            Err(AppError::RemoteServiceUnavailable(
                "injected failure for fallback test".to_string(),
            ))
        }

        async fn generate_tag(&self, _model: &str, _prompt: &str) -> Result<String> {
            Err(AppError::RemoteServiceUnavailable(
                "injected failure for fallback test".to_string(),
            ))
        }
    }

    #[tokio::test]
    async fn main_llm_failure_returns_fallback_reply_and_keeps_previous_emotion() {
        let tmp = tempfile::tempdir().expect("roles tempdir");
        let role_dir = tmp.path().join("fail_role");
        std::fs::create_dir_all(&role_dir).expect("role dir");

        let role = Role {
            id: "fail_role".to_string(),
            name: "Fail Role".to_string(),
            description: "role used to verify LLM failure fallback".to_string(),
            version: "1.0.0".to_string(),
            author: "kernel-test".to_string(),
            core_personality: "test role".to_string(),
            user_relations: vec![UserRelation {
                id: "friend".to_string(),
                name: "friend".to_string(),
                prompt_hint: String::new(),
                favor_multiplier: 1.0,
                initial_favorability: 50.0,
            }],
            default_relation: "friend".to_string(),
            ollama_model: Some("test-model:latest".to_string()),
            ..Role::default()
        };

        std::fs::write(
            role_dir.join("manifest.json"),
            serde_json::to_string_pretty(&disk_manifest_from_role(&role)).expect("manifest json"),
        )
        .expect("write manifest");
        std::fs::write(
            role_dir.join("settings.json"),
            serde_json::to_string_pretty(&disk_role_settings_from_role(&role))
                .expect("settings json"),
        )
        .expect("write settings");

        let state = AppState::new_in_memory_with_llm(Arc::new(FailingLlmClient), tmp.path())
            .await
            .expect("test app state");

        let srid = "fail_role";
        state
            .db_manager
            .ensure_role_runtime(srid)
            .await
            .expect("role runtime");
        state
            .db_manager
            .set_current_emotion(srid, "sad")
            .await
            .expect("baseline emotion");

        let response = super::process_message(
            &state,
            &SendMessageRequest {
                role_id: srid.to_string(),
                user_message: "Hello there".to_string(),
                scene_id: None,
                session_id: None,
                include_raw_reply: None,
                adult: None,
            },
        )
        .await
        .expect("turn must degrade gracefully instead of erroring");

        assert!(
            response.reply_is_fallback,
            "LLM failure must be reported via reply_is_fallback"
        );
        assert!(
            response.llm_fallback_reason.is_some(),
            "LLM failure must carry a fallback reason"
        );
        assert!(
            !response.reply.trim().is_empty(),
            "fallback reply must not be empty"
        );
        assert_ne!(response.reply, "Hello there");
        assert_eq!(
            state
                .db_manager
                .get_current_emotion(srid)
                .await
                .expect("read emotion"),
            Some("sad".to_string()),
            "degraded turn must keep the previous emotion (B M1 slice 2)"
        );
    }
}

//! Orchestration for cancellable background adult-beat generation.
//!
//! Generation is speculative. Only `commit_adult_staged_beat` may make a beat
//! visible to future turns by appending short-term context and chat history.

use crate::domain::chat_engine::context::validate_scene_id;
use crate::domain::chat_engine::{conversation_state_role_id, process_message};
use crate::domain::ports::conversation_persist::{TurnAutoCleanupConfig, TurnPersistRequest};
use crate::error::{AppError, Result};
use crate::models::dto::{
    AdultInteractionAction, AdultStageDirective, AdultStagedBeatDto,
    BeginAdultStageGenerationRequest, BeginAdultStageGenerationResponse,
    CancelAdultStageGenerationRequest, CommitAdultStagedBeatRequest, ListAdultStagedBeatsRequest,
    ListAdultStagedBeatsResponse, SendMessageRequest, SendMessageResponse, StageAdultBeatRequest,
};
use crate::models::ADULT_BACKGROUND_QUEUE_CAP_MAX;
use crate::state::AppState;

pub const ADULT_CONTINUATION_INPUT: &str =
    "[System: continue the current interaction by one beat. Do not invent the user's words, actions, choices, or feelings.]";

fn normalized_scene(scene_id: Option<&str>) -> String {
    scene_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("default")
        .to_string()
}

fn canonical_scene(role_id: &str, role: &crate::models::Role, scene_id: Option<&str>) -> String {
    validate_scene_id(role_id, &role.scene_ids, normalized_scene(scene_id))
}

fn validate_open_gates(request: &crate::models::dto::AdultInteractionRequest) -> Result<()> {
    if request.gates_open() {
        Ok(())
    } else {
        Err(AppError::InvalidParameter(
            "adult stage requires all adult gates to be open".to_string(),
        ))
    }
}

async fn validate_role(
    state: &AppState,
    role_id: &str,
) -> Result<std::sync::Arc<crate::models::Role>> {
    let role = state.load_role_cached_async(role_id).await?;
    if role.adult_extension.is_none() {
        return Err(AppError::InvalidParameter(
            "role has no adult extension".to_string(),
        ));
    }
    Ok(role)
}

/// Starts a cancellable staged-beat generation for one role/chat/scene.
///
/// # Errors
///
/// Returns an error when adult gates fail, the role extension cannot be
/// loaded, or durable generation state cannot be created.
pub async fn begin_adult_stage_generation(
    state: &AppState,
    request: BeginAdultStageGenerationRequest,
) -> Result<BeginAdultStageGenerationResponse> {
    validate_open_gates(&request.adult)?;
    let role_id = request.role_id.trim();
    let role = validate_role(state, role_id).await?;
    let scene_id = canonical_scene(role_id, role.as_ref(), request.scene_id.as_deref());
    let srid = conversation_state_role_id(role_id, request.session_id.as_deref());
    let _stage_guard = state
        .adult_stage_lock_for(srid.as_str(), scene_id.as_str())
        .lock_owned()
        .await;
    let (generation_id, invalidated_generation_ids) = state
        .db_manager
        .begin_adult_stage_generation(srid.as_str(), role_id, scene_id.as_str())
        .await?;
    for invalidated in invalidated_generation_ids {
        state.cancel_adult_stage_generation_in_flight(invalidated.as_str());
    }
    let _ = state.register_adult_stage_generation(generation_id.as_str());
    if !state
        .db_manager
        .adult_stage_generation_active(
            generation_id.as_str(),
            srid.as_str(),
            role_id,
            scene_id.as_str(),
        )
        .await?
    {
        state.cancel_adult_stage_generation_in_flight(generation_id.as_str());
        return Err(AppError::InvalidParameter(
            "adult stage generation was superseded during startup".to_string(),
        ));
    }
    Ok(BeginAdultStageGenerationResponse {
        generation_id,
        next_sequence: 0,
    })
}

/// Generates one speculative structured beat without committing turn effects.
///
/// # Errors
///
/// Returns an error for invalid gates, generation or sequence state,
/// cancellation, model failure, malformed structured output, or stage storage.
pub async fn generate_adult_staged_beat(
    state: &AppState,
    request: StageAdultBeatRequest,
) -> Result<AdultStagedBeatDto> {
    validate_open_gates(&request.adult)?;
    let role_id = request.role_id.trim();
    let role = validate_role(state, role_id).await?;
    let scene_id = canonical_scene(role_id, role.as_ref(), request.scene_id.as_deref());
    let srid = conversation_state_role_id(role_id, request.session_id.as_deref());
    let _stage_guard = state
        .adult_stage_lock_for(srid.as_str(), scene_id.as_str())
        .lock_owned()
        .await;
    let generation_state = state
        .db_manager
        .adult_stage_generation_state_for_chat(
            request.generation_id.as_str(),
            srid.as_str(),
            role_id,
            scene_id.as_str(),
        )
        .await?;
    if !matches!(
        generation_state,
        Some((true, next_sequence)) if next_sequence == request.sequence
    ) {
        return Err(AppError::InvalidParameter(
            "adult stage generation is inactive or sequence is no longer current".to_string(),
        ));
    }
    let buffered = state
        .db_manager
        .buffered_adult_stage_beat_count(request.generation_id.as_str())
        .await?;
    if buffered >= ADULT_BACKGROUND_QUEUE_CAP_MAX {
        return Err(AppError::InvalidParameter(format!(
            "adult staged beat buffer reached the hard limit of {ADULT_BACKGROUND_QUEUE_CAP_MAX}"
        )));
    }

    let signal = state.adult_stage_cancellation(request.generation_id.as_str());
    let mut adult = request.adult;
    adult.action = AdultInteractionAction::Continue;
    adult.interaction_active = true;
    adult.stage = Some(AdultStageDirective {
        generation_id: request.generation_id.clone(),
        sequence: request.sequence,
    });
    let send = SendMessageRequest {
        role_id: role_id.to_string(),
        user_message: ADULT_CONTINUATION_INPUT.to_string(),
        scene_id: Some(scene_id.clone()),
        session_id: request.session_id.clone(),
        include_raw_reply: None,
        adult: Some(adult),
    };
    let started_at = std::time::Instant::now();
    let response = tokio::select! {
        biased;
        () = signal.cancelled() => {
            return Err(AppError::InvalidParameter(
                "adult stage generation cancelled".to_string(),
            ));
        }
        result = process_message(state, &send) => result?,
    };
    let beat = response.adult_beat.as_ref().ok_or_else(|| {
        AppError::InvalidParameter("staged response did not contain a structured beat".to_string())
    })?;
    let transcript = crate::domain::adult_interaction::transcript_reply(beat);
    let model_name = crate::domain::effective_llm_model::resolve_effective_ollama_model(
        state,
        role.as_ref(),
        srid.as_str(),
    )
    .await
    .ok();
    state
        .db_manager
        .store_adult_staged_beat(
            request.generation_id.as_str(),
            srid.as_str(),
            role_id,
            scene_id.as_str(),
            request.sequence,
            &response,
            transcript.as_str(),
            model_name.as_deref(),
            u64::try_from(started_at.elapsed().as_millis()).unwrap_or(u64::MAX),
        )
        .await?;
    Ok(AdultStagedBeatDto {
        generation_id: request.generation_id,
        sequence: request.sequence,
        response,
    })
}

/// Commits one staged beat in order to adult short-term memory and chat history.
///
/// # Errors
///
/// Returns an error when the beat does not belong to this chat, is out of
/// order/cancelled, or its durable memory/finalization writes fail.
pub async fn commit_adult_staged_beat(
    state: &AppState,
    request: CommitAdultStagedBeatRequest,
) -> Result<SendMessageResponse> {
    let role_id = request.role_id.trim();
    let role = state.load_role_cached_async(role_id).await?;
    let scene_id = canonical_scene(role_id, role.as_ref(), request.scene_id.as_deref());
    let srid = conversation_state_role_id(role_id, request.session_id.as_deref());
    let _stage_guard = state
        .adult_stage_lock_for(srid.as_str(), scene_id.as_str())
        .lock_owned()
        .await;
    let _guard = state.turn_lock_for(srid.as_str()).lock_owned().await;
    let mut stored = state
        .db_manager
        .load_adult_staged_beat(request.generation_id.as_str(), request.sequence)
        .await?
        .ok_or_else(|| AppError::InvalidParameter("staged beat not found".to_string()))?;
    let turn_policies = state.turn_policies_for_scene(Some(scene_id.as_str()));
    state
        .db_manager
        .commit_adult_staged_short_term(
            &stored,
            srid.as_str(),
            scene_id.as_str(),
            ADULT_CONTINUATION_INPUT,
            turn_policies.memory_fifo_limit,
        )
        .await?;

    let append = state
        .conversation_persist_port()
        .append_turn(TurnPersistRequest {
            idempotency_key: Some(format!("{}:{}", request.generation_id, request.sequence)),
            session_id: srid.clone(),
            role_id: role_id.to_string(),
            scene_id: scene_id.clone(),
            user_message: ADULT_CONTINUATION_INPUT.to_string(),
            user_message_hidden: true,
            assistant_reply: stored.transcript_reply.clone(),
            reply_is_fallback: stored.response.reply_is_fallback,
            model_name: stored.model_name.clone(),
            response_ms: stored.response_ms,
            user_emotion: Some("neutral".to_string()),
            bot_emotion: stored.bot_emotion.clone(),
            bot_emotion_source: None,
            bot_emotion_labels: vec![],
            user_emotion_scores: None,
            emotion_pattern: None,
            emotion_confidence: None,
            emotion_intensity: None,
            emotion_dissonance: None,
            emotion_hint: None,
            reply_segments: None,
            reply_segment_delays_ms: None,
            max_messages_per_session: role.pack_chat_storage_config.max_messages_per_session,
            auto_cleanup_config: TurnAutoCleanupConfig::from_role_config(
                &role.pack_chat_storage_config,
            ),
            chat_storage_location: role.pack_chat_storage_config.location.clone(),
        })
        .await;
    match append {
        Ok(ids) => {
            stored.response.user_message_id = Some(ids.user_message_id);
            stored.response.assistant_message_id = Some(ids.assistant_message_id);
            stored.response.user_message_timestamp = Some(ids.user_message_timestamp);
            stored.response.assistant_message_timestamp = Some(ids.assistant_message_timestamp);
            stored.response.chat_persist_failed = None;
            stored.response.chat_persist_error = None;
        }
        Err(error) => {
            stored.response.chat_persist_failed = Some(true);
            stored.response.chat_persist_error = Some(error.to_string());
        }
    }
    let ended = stored.response.adult_beat.as_ref().is_some_and(|beat| {
        matches!(
            beat.interaction_state,
            crate::models::dto::AdultInteractionState::Ended
        )
    });
    state
        .db_manager
        .finish_adult_staged_beat(request.generation_id.as_str(), request.sequence, ended)
        .await?;
    if ended {
        state.finish_adult_stage_generation_in_flight(request.generation_id.as_str());
    }
    Ok(stored.response)
}

/// Cancels generation and discards all pending, uncommitted beats.
///
/// # Errors
///
/// Returns an error when durable cancellation state cannot be updated.
pub async fn cancel_adult_stage_generation(
    state: &AppState,
    request: CancelAdultStageGenerationRequest,
) -> Result<()> {
    let role_id = request.role_id.trim();
    let role = state.load_role_cached_async(role_id).await?;
    let scene_id = canonical_scene(role_id, role.as_ref(), request.scene_id.as_deref());
    let srid = conversation_state_role_id(role_id, request.session_id.as_deref());
    state.cancel_adult_stage_generation_in_flight(request.generation_id.as_str());
    let _stage_guard = state
        .adult_stage_lock_for(srid.as_str(), scene_id.as_str())
        .lock_owned()
        .await;
    state
        .db_manager
        .cancel_adult_stage_generation(
            request.generation_id.as_str(),
            srid.as_str(),
            role_id,
            scene_id.as_str(),
        )
        .await?;
    Ok(())
}

/// Restores ordered, not-yet-finalized beats for one generation.
///
/// # Errors
///
/// Returns an error when the generation is unknown or its durable state cannot
/// be read.
pub async fn list_adult_staged_beats(
    state: &AppState,
    request: ListAdultStagedBeatsRequest,
) -> Result<ListAdultStagedBeatsResponse> {
    let role_id = request.role_id.trim();
    let role = state.load_role_cached_async(role_id).await?;
    let scene_id = canonical_scene(role_id, role.as_ref(), request.scene_id.as_deref());
    let srid = conversation_state_role_id(role_id, request.session_id.as_deref());
    let state_row = state
        .db_manager
        .adult_stage_generation_state_for_chat(
            request.generation_id.as_str(),
            srid.as_str(),
            role_id,
            scene_id.as_str(),
        )
        .await?
        .ok_or_else(|| {
            AppError::InvalidParameter("adult stage generation not found for this chat".to_string())
        })?;
    let beats = state
        .db_manager
        .list_adult_staged_beats(request.generation_id.as_str())
        .await?;
    Ok(ListAdultStagedBeatsResponse {
        generation_id: request.generation_id,
        active: state_row.0,
        next_sequence: state_row.1,
        beats,
    })
}

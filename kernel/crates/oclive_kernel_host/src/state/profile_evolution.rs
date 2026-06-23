//! Background mutable-profile LLM evolution (off the chat turn critical path).

use crate::domain::mutable_profile_llm;
use crate::domain::ports::LlmClient;
use crate::models::{EventType, PersonalityVector, Role};
use crate::state::{AppState, SessionCache};
use std::sync::{Arc, LazyLock};
use tokio::sync::Semaphore;

static MUTABLE_PROFILE_EVOLUTION_SEM: LazyLock<Semaphore> = LazyLock::new(|| Semaphore::new(2));

/// Profile mutable-personality LLM + DB writes run off the critical path; next turn reads from DB.
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_mutable_profile_evolution(
    state: &AppState,
    primary_llm: Arc<dyn LlmClient>,
    role: Arc<Role>,
    srid: String,
    path_label: String,
    ollama_model: String,
    user_message: String,
    reply: String,
    user_emotion: String,
    event_type: EventType,
    impact_scaled: f64,
) {
    let db = Arc::clone(&state.db_manager);
    let session_cache: Arc<SessionCache> = Arc::clone(&state.session_cache);
    tokio::spawn(async move {
        let Ok(_permit) = MUTABLE_PROFILE_EVOLUTION_SEM.acquire().await else {
            return;
        };
        let prev = match db.get_mutable_personality(&srid).await {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!(
                    target: "oclive_chat",
                    role_id = %srid,
                    error = %e,
                    "background mutable_profile: get_mutable_personality failed"
                );
                return;
            }
        };
        let next = match mutable_profile_llm::evolve_mutable_personality_with_llm(
            &primary_llm,
            ollama_model.as_str(),
            mutable_profile_llm::MutableEvolutionInput {
                role_name: role.name.as_str(),
                core_personality: role.core_personality.as_str(),
                prev_mutable: prev.as_str(),
                user_message: user_message.as_str(),
                bot_reply: reply.as_str(),
                user_emotion: user_emotion.as_str(),
                event_type: &event_type,
                impact_scaled,
                evolution: &role.evolution_config,
            },
        )
        .await
        {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!(
                    target: "oclive_chat",
                    path_label = %path_label,
                    role_id = %srid,
                    error = %e,
                    "background mutable_profile_llm failed; keeping previous archive"
                );
                return;
            }
        };
        if let Err(e) = db.set_mutable_personality(&srid, &next).await {
            tracing::warn!(
                target: "oclive_chat",
                role_id = %srid,
                error = %e,
                "background mutable_profile: set_mutable_personality failed"
            );
            return;
        }
        let core_v = PersonalityVector::from(&role.default_personality);
        let personality_after =
            crate::domain::profile_personality::effective_vector_from_profile(&role, &next);
        let delta_out = PersonalityVector::sub_components(&personality_after, &core_v);
        if let Err(e) = db
            .set_core_delta_personality_json(&srid, &core_v.to_json_vec(), &delta_out.to_json_vec())
            .await
        {
            tracing::warn!(
                target: "oclive_chat",
                role_id = %srid,
                error = %e,
                "background mutable_profile: set_core_delta_personality_json failed"
            );
            return;
        }
        session_cache
            .personality_cache()
            .set(srid, personality_after);
    });
}

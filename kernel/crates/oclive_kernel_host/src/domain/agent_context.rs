//! Build [`AgentInput`] with role constraints and MCP tool schemas.

use crate::domain::chat_engine::turn_prefetch::TurnPrefetch;
use crate::error::Result;
use crate::models::{PersonalityVector, Role};
use crate::state::AppState;
use oclive_kernel_contracts::McpBridgePort;
use oclive_kernel_types::{AgentInput, AgentRoleConstraints, AgentTurnContext};

const RECENT_TURN_LIMIT: usize = 2;

fn personality_to_f32_vec(p: &PersonalityVector) -> Vec<f32> {
    p.to_vec7()
        .into_iter()
        .map(|v| v as f32)
        .collect::<Vec<f32>>()
}

fn default_personality_f32(role: &Role) -> Vec<f32> {
    personality_to_f32_vec(&PersonalityVector::from(&role.default_personality))
}

/// Assemble agent turn input including B-tier constraints (personality + relation + scene).
///
/// When `prefetch` is supplied, reuses turn-wide recent context and user identity (no duplicate DB reads).
///
/// # Errors
///
/// Database or identity resolution failures.
#[allow(clippy::too_many_arguments)]
pub async fn build_agent_input(
    state: &AppState,
    role: &Role,
    srid: &str,
    scene_id: &str,
    message: &str,
    model: &str,
    bridge: &dyn McpBridgePort,
    prefetch: Option<&TurnPrefetch>,
) -> Result<AgentInput> {
    let (user_relation_key, recent_turns) = if let Some(pf) = prefetch {
        (
            pf.resolved_identity.relation_key.clone(),
            pf.recent_turns
                .iter()
                .take(RECENT_TURN_LIMIT)
                .cloned()
                .collect(),
        )
    } else {
        let resolved_identity = crate::domain::user_identity_loader::resolve_active_user_identity(
            state,
            role,
            srid,
            Some(scene_id),
        )
        .await?;
        let (turns, _, _) =
            crate::domain::chat_engine::context::load_recent_context(state, srid).await?;
        (
            resolved_identity.relation_key,
            turns.into_iter().take(RECENT_TURN_LIMIT).collect(),
        )
    };

    let (personality_row, favorability, relation_state, tools) = tokio::try_join!(
        state.db_manager.get_latest_personality_vector(srid),
        state
            .db_manager
            .favorability_for_identity_with_runtime_fallback(srid, user_relation_key.as_str()),
        async {
            let rel_id = state
                .db_manager
                .get_relation_state_for_identity(srid, user_relation_key.as_str())
                .await?;
            let rel_global = state.db_manager.get_relation_state(srid).await?;
            Ok::<String, crate::error::AppError>(
                rel_id
                    .or(rel_global)
                    .unwrap_or_else(|| "Stranger".to_string()),
            )
        },
        async { bridge.list_agent_tool_schemas().await },
    )?;

    let personality_vector = personality_row
        .as_ref()
        .map(personality_to_f32_vec)
        .unwrap_or_else(|| default_personality_f32(role));

    let scene_label = state.storage.scene_display_name_for_role(role, scene_id);

    Ok(AgentInput {
        role_id: role.id.clone(),
        session_namespace: srid.to_string(),
        message: message.to_string(),
        model: model.to_string(),
        scene_id: scene_id.to_string(),
        constraints: AgentRoleConstraints {
            personality_vector,
            relation_state,
            favorability,
            scene_label,
            interaction_mode: role.interaction_mode.clone(),
            policy_text: None,
        },
        tools,
        turn_context: AgentTurnContext {
            recent_turns,
            tool_results: Vec::new(),
        },
        protocol_version: 1,
    })
}

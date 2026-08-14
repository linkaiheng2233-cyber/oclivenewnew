//! Role read-model assembly for `RoleData` / `RoleInfo` (shared by load_role and get_role_info).

use crate::command_error::CommandError;
use crate::domain::chat_engine::conversation_state_role_id;
use crate::domain::effective_llm_model::resolve_effective_ollama_model;
use crate::domain::reply_post_processor::apply_effective_post_processor_config;
use crate::error::AppError;
use crate::models::dto::{DisplayMetricsDto, RoleData, RoleInfo, SceneLabelEntry};
use crate::models::plugin_backends::{
    AgentBackend, EmotionBackend, EventBackend, LlmBackend, MemoryBackend, PluginBackendsOverride,
    PromptBackend,
};
use crate::models::role::Role;
use crate::models::ReplyPostProcessorBackendKind;
use crate::service::role::interaction::resolve_interaction_ui_snapshot;
use crate::service::role::runtime::{
    current_favorability_for_effective_identity, maybe_seed_initial_favorability_with_extras,
    resolve_relation_state_for_ui, role_runtime_extras,
};
use crate::state::AppState;
use serde::de::DeserializeOwned;
use serde_json::Value;

type SlotRegistryMap = std::collections::BTreeMap<String, oclive_validation::SlotRegistryEntry>;

/// UI-only affect snapshot (shared by role info, GET display metrics, and affect push).
#[must_use]
pub fn build_display_metrics(
    favor: f64,
    relation_state: &str,
    personality: &crate::models::PersonalityVector,
) -> DisplayMetricsDto {
    DisplayMetricsDto {
        favor,
        relation_summary: relation_state.to_string(),
        traits: personality.to_vec7(),
    }
}

fn reply_post_processor_role_info_fields(
    state: &AppState,
    role: &Role,
) -> (bool, String, Option<String>) {
    if !role.pack_reply_post_processor_config.enabled {
        return (false, "off".into(), None);
    }
    let eff = apply_effective_post_processor_config(
        state.host_profile.as_ref(),
        &role.pack_reply_post_processor_config,
    );
    let backend = match eff.backend {
        ReplyPostProcessorBackendKind::Builtin => "builtin",
        ReplyPostProcessorBackendKind::Remote => "remote",
        ReplyPostProcessorBackendKind::Directory => "directory",
    }
    .to_string();
    (true, backend, Some(eff.builtin.profile))
}

fn session_namespace(role_id: &str, session_id: Option<&str>) -> String {
    conversation_state_role_id(role_id, session_id)
}

fn slot_registry_role_info_fields(
    state: &AppState,
    role: &Role,
    session_ns: &str,
) -> (
    Option<SlotRegistryMap>,
    Option<SlotRegistryMap>,
    Vec<String>,
) {
    let pack = role.slot_registry.clone();
    let effective = state.effective_slot_registry_for_session(role, session_ns);
    let keys = state.slot_session_overridden_keys(session_ns);
    (pack, effective, keys)
}

fn blueprint_groups_pack(
    role: &Role,
) -> Option<std::collections::BTreeMap<String, oclive_validation::SlotGroupEntry>> {
    role.slot_groups.as_ref().filter(|m| !m.is_empty()).cloned()
}

/// Folds session `slot_registry` overrides into a C1-compatible six-slot `PluginBackendsOverride` (read-only display).
pub fn plugin_backends_override_from_slot_session(
    state: &AppState,
    role: &Role,
    session_namespace: &str,
) -> Option<PluginBackendsOverride> {
    let pack = role.slot_registry.as_ref()?;
    let ov = state.session_slot_overrides(session_namespace);
    if ov.is_empty() {
        return None;
    }
    let mut out = PluginBackendsOverride::default();
    for (key, patch) in ov {
        let Some(entry) = pack.get(&key) else {
            continue;
        };
        if let Some(ref b) = patch.backend {
            let wire = b.trim();
            if wire.is_empty() {
                continue;
            }
            match entry.slot_type.as_str() {
                "memory" => {
                    out.memory = parse_backend_wire::<MemoryBackend>("memory", wire).ok();
                }
                "emotion" => {
                    out.emotion = parse_backend_wire::<EmotionBackend>("emotion", wire).ok();
                }
                "event" => {
                    out.event = parse_backend_wire::<EventBackend>("event", wire).ok();
                }
                "prompt" => {
                    out.prompt = parse_backend_wire::<PromptBackend>("prompt", wire).ok();
                }
                "llm" => {
                    out.llm = parse_backend_wire::<LlmBackend>("llm", wire).ok();
                }
                "agent" => {
                    out.agent = parse_backend_wire::<AgentBackend>("agent", wire).ok();
                }
                _ => {}
            }
        }
        if entry.slot_type == "memory" {
            if let Some(ref id) = patch.local_memory_provider_id {
                let t = id.trim();
                out.local_memory_provider_id = if t.is_empty() {
                    None
                } else {
                    Some(t.to_string())
                };
            }
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

fn parse_backend_wire<T: DeserializeOwned>(module: &str, value: &str) -> Result<T, CommandError> {
    let t = value.trim();
    if t.is_empty() {
        return Err(AppError::InvalidParameter(format!(
            "session backend override: module={} backend must not be empty",
            module
        ))
        .into());
    }
    Ok(
        serde_json::from_value::<T>(Value::String(t.to_string())).map_err(|_| {
            AppError::InvalidParameter(format!(
                "session backend override: module={} backend={} is invalid",
                module, t
            ))
        })?,
    )
}

/// Gathers runtime fields and builds [`RoleData`] (no load-role side effects).
///
/// # Errors
///
/// Returns [`Err`] when DB reads or runtime resolution fail.
pub async fn assemble_role_data(
    state: &AppState,
    role_id: &str,
    role: &Role,
) -> Result<RoleData, CommandError> {
    let personality = state.get_current_personality(role_id, role).await?;

    let current_scene = state.db_manager.get_current_scene(role_id).await?;
    let rt = role_runtime_extras(state, role_id, current_scene.as_deref(), role).await?;
    maybe_seed_initial_favorability_with_extras(state, role_id, role, &rt).await?;
    let current_favorability = current_favorability_for_effective_identity(
        state,
        role_id,
        rt.current_user_relation.as_str(),
    )
    .await?;

    let memory_count = state.memory_repo.count_memories(role_id).await?;

    let event_count = state.db_manager.count_events(role_id).await?;
    let session_ns = session_namespace(role_id, None);
    let effective_ollama_model =
        resolve_effective_ollama_model(state, role, session_ns.as_str()).await?;
    let relation_state =
        resolve_relation_state_for_ui(state, role_id, rt.current_user_relation.as_str()).await?;
    let remote_life_enabled = state.db_manager.get_remote_life_enabled(role_id).await?;
    let remote_life_pack_default = role
        .remote_presence
        .as_ref()
        .and_then(|r| r.default_enabled);

    let virtual_time_ms = state
        .db_manager
        .get_virtual_time_ms(role_id)
        .await?
        .unwrap_or(0);
    let interaction =
        resolve_interaction_ui_snapshot(state, role_id, role, virtual_time_ms).await?;

    let plugin_backends_session_override =
        plugin_backends_override_from_slot_session(state, role, session_ns.as_str());
    let plugin_backends_effective =
        state.effective_plugin_backends_for_session(role, session_ns.as_str());
    let plugin_backends_effective_sources =
        state.effective_plugin_backend_sources_for_session(role, session_ns.as_str());
    let (slot_registry_pack, slot_registry_effective, slot_session_overridden_keys) =
        slot_registry_role_info_fields(state, role, session_ns.as_str());

    Ok(RoleData {
        role_id: role_id.to_string(),
        name: role.name.clone(),
        version: role.version.clone(),
        author: role.author.clone(),
        description: role.description.clone(),
        adult_extension_available: role.adult_extension.is_some(),
        adult_extension_error: role.adult_extension_error.clone(),
        personality_vector: personality.to_vec7(),
        current_favorability,
        display_metrics: Some(build_display_metrics(
            current_favorability,
            relation_state.as_str(),
            &personality,
        )),
        current_emotion: state
            .db_manager
            .get_current_emotion(role_id)
            .await?
            .unwrap_or_else(|| "Neutral".to_string()),
        memory_count: memory_count as i32,
        event_count: event_count as i32,
        user_relations: rt.user_relations,
        default_relation: rt.default_relation,
        relation_state,
        current_user_relation: rt.current_user_relation.clone(),
        use_manifest_default: rt.use_manifest_default,
        remote_life_enabled,
        remote_life_pack_default,
        event_impact_factor: rt.event_impact_factor,
        personality_source: role.evolution_config.personality_source,
        effective_ollama_model,
        identity_binding: role.identity_binding,
        interaction_mode: interaction.mode_str,
        interaction_mode_pack_default: interaction.pack_default,
        current_life: interaction.current_life,
        plugin_backends: role.plugin_backends.as_ref().clone(),
        plugin_backends_session_override,
        plugin_backends_effective: plugin_backends_effective.as_ref().clone(),
        plugin_backends_effective_sources,
        pack_ui_config: role.ui_config.clone(),
        pack_ui_baseline: role.plugin_state_ui_baseline().clone(),
        author_pack: role.author_pack.clone(),
        slot_registry_pack,
        slot_registry_effective,
        slot_session_overridden_keys,
        blueprint_groups_pack: blueprint_groups_pack(role),
    })
}

/// Gathers runtime fields and builds [`RoleInfo`] (caller must ensure role runtime exists).
///
/// # Errors
///
/// Returns [`Err`] when DB reads or runtime resolution fail.
pub async fn assemble_role_info(
    state: &AppState,
    role_id: &str,
    role: &Role,
    session_id: Option<&str>,
) -> Result<RoleInfo, CommandError> {
    let session_ns = session_namespace(role_id, session_id);

    let plugin_backends_session_override =
        plugin_backends_override_from_slot_session(state, role, session_ns.as_str());
    let plugin_backends_effective =
        state.effective_plugin_backends_for_session(role, session_ns.as_str());
    let plugin_backends_effective_sources =
        state.effective_plugin_backend_sources_for_session(role, session_ns.as_str());

    let current_scene = state.db_manager.get_current_scene(role_id).await?;
    let user_presence_scene = state.db_manager.get_user_presence_scene(role_id).await?;
    let rt = role_runtime_extras(state, role_id, current_scene.as_deref(), role).await?;
    maybe_seed_initial_favorability_with_extras(state, role_id, role, &rt).await?;

    let personality = state.get_current_personality(role_id, role).await?;

    let last_interaction = state
        .db_manager
        .get_latest_memory_created_at(role_id)
        .await?
        .map(|t| t.to_rfc3339());

    let scenes = state.storage.list_scene_ids(role_id)?;
    let scene_labels: Vec<SceneLabelEntry> = scenes
        .iter()
        .map(|id| SceneLabelEntry {
            id: id.clone(),
            label: state.storage.scene_display_name_for_role(role, id),
        })
        .collect();
    let virtual_time_ms = state
        .db_manager
        .get_virtual_time_ms(role_id)
        .await?
        .unwrap_or(0);
    let current_favorability = current_favorability_for_effective_identity(
        state,
        role_id,
        rt.current_user_relation.as_str(),
    )
    .await?;
    let effective_ollama_model =
        resolve_effective_ollama_model(state, role, session_ns.as_str()).await?;
    let relation_state =
        resolve_relation_state_for_ui(state, role_id, rt.current_user_relation.as_str()).await?;
    let remote_life_enabled = state.db_manager.get_remote_life_enabled(role_id).await?;
    let remote_life_pack_default = role
        .remote_presence
        .as_ref()
        .and_then(|r| r.default_enabled);

    let interaction =
        resolve_interaction_ui_snapshot(state, role_id, role, virtual_time_ms).await?;

    let (knowledge_enabled, knowledge_chunk_count) = match &role.knowledge_index {
        Some(idx) => (true, idx.chunks.len() as i32),
        None => (false, 0),
    };

    let (slot_registry_pack, slot_registry_effective, slot_session_overridden_keys) =
        slot_registry_role_info_fields(state, role, session_ns.as_str());

    let (reply_post_processor_enabled, reply_post_processor_backend, reply_post_processor_profile) =
        reply_post_processor_role_info_fields(state, role);

    Ok(RoleInfo {
        role_id: role_id.to_string(),
        role_name: role.name.clone(),
        version: role.version.clone(),
        author: role.author.clone(),
        description: role.description.clone(),
        adult_extension_available: role.adult_extension.is_some(),
        adult_extension_error: role.adult_extension_error.clone(),
        current_favorability,
        display_metrics: Some(build_display_metrics(
            current_favorability,
            relation_state.as_str(),
            &personality,
        )),
        current_emotion: state
            .db_manager
            .get_current_emotion(role_id)
            .await?
            .unwrap_or_else(|| "Neutral".to_string()),
        personality_vector: personality.to_vec7(),
        personality_source: role.evolution_config.personality_source,
        last_interaction,
        scenes,
        scene_labels,
        current_scene,
        user_presence_scene,
        virtual_time_ms,
        user_relations: rt.user_relations,
        default_relation: rt.default_relation,
        current_user_relation: rt.current_user_relation.clone(),
        use_manifest_default: rt.use_manifest_default,
        relation_state,
        remote_life_enabled,
        remote_life_pack_default,
        event_impact_factor: rt.event_impact_factor,
        effective_ollama_model,
        identity_binding: role.identity_binding,
        interaction_mode: interaction.mode_str,
        interaction_mode_pack_default: interaction.pack_default,
        current_life: interaction.current_life,
        plugin_backends: role.plugin_backends.as_ref().clone(),
        plugin_backends_session_override,
        plugin_backends_effective: plugin_backends_effective.as_ref().clone(),
        plugin_backends_effective_sources,
        knowledge_enabled,
        knowledge_chunk_count,
        pack_ui_config: role.ui_config.clone(),
        pack_ui_baseline: role.plugin_state_ui_baseline().clone(),
        author_pack: role.author_pack.clone(),
        slot_registry_pack,
        slot_registry_effective,
        slot_session_overridden_keys,
        blueprint_groups_pack: blueprint_groups_pack(role),
        dual_core_enabled: role.dual_core_gated(),
        pipeline_experimental_actions: role
            .pipeline_experimental
            .as_ref()
            .map(|steps| steps.iter().map(|s| s.action.clone()).collect())
            .unwrap_or_default(),
        reply_post_processor_enabled,
        reply_post_processor_backend,
        reply_post_processor_profile,
    })
}

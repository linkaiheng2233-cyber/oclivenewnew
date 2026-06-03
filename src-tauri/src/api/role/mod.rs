//! Role API: manifest loading, runtime snapshots, identity, evolution factors, and related Tauri commands.
//!
//! Blueprint v2 disk writes go through [`save_role_slot_registry`]; there is **no** legacy Tauri command that only writes
//! `manifest.json`/`settings.json` `plugin_backends` (old packs are read-only via [`RoleStorage::load_role_from_legacy_manifest_dir`]).

pub mod display;
pub mod expert;
pub mod evolution;
pub mod interaction;
pub mod relation;
pub mod runtime;
pub mod slot_session;

pub use evolution::{
    set_evolution_factor_impl, set_remote_life_enabled_impl, set_role_interaction_mode_impl,
};
pub use relation::{
    clear_scene_user_relation_impl, set_scene_user_relation_impl, set_user_relation_impl,
};
pub use expert::{get_expert_routing, list_blueprint_includes, save_expert_routing};
pub use slot_session::{
    clear_all_session_slot_overrides_impl, clear_session_slot_override_impl,
    get_plugin_resolution_debug_impl, save_role_slot_registry_impl,
    set_session_plugin_backend_impl, set_session_slot_override_impl,
};
pub(crate) use slot_session::build_plugin_resolution_debug_info;

use crate::error::AppError;
use crate::api::error::CommandError;
use crate::models::dto::{
    GetRoleInfoRequest, RoleData, RoleInfo, RoleSummary, SceneLabelEntry,
};
use crate::models::plugin_backends::{
    AgentBackend, EmotionBackend, EventBackend, LlmBackend, MemoryBackend, PluginBackendsOverride,
    PromptBackend,
};
use crate::models::role::Role;
use crate::state::{AppState, SharedAppState};
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};

use interaction::resolve_interaction_ui_snapshot;
use runtime::{
    current_favorability_for_effective_identity, maybe_seed_initial_favorability_with_extras,
    resolve_relation_state_for_ui, role_runtime_extras,
};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};

pub(crate) const EVENT_IMPACT_MIN: f64 = 0.05;
pub(crate) const EVENT_IMPACT_MAX: f64 = 5.0;

pub(crate) fn session_namespace(role_id: &str, session_id: Option<&str>) -> String {
    crate::domain::chat_engine::conversation_state_role_id(role_id, session_id)
}

type SlotRegistryMap = std::collections::BTreeMap<String, oclive_validation::SlotRegistryEntry>;

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

fn blueprint_groups_pack(role: &Role) -> Option<std::collections::BTreeMap<String, oclive_validation::SlotGroupEntry>> {
    role.slot_groups
        .as_ref()
        .filter(|m| !m.is_empty())
        .cloned()
}

/// Folds session `slot_registry` overrides into a C1-compatible six-slot `PluginBackendsOverride` (read-only display).
fn plugin_backends_override_from_slot_session(
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
    Ok(serde_json::from_value::<T>(Value::String(t.to_string())).map_err(|_| {
        AppError::InvalidParameter(format!(
            "session backend override: module={} backend={} is invalid",
            module, t
        ))
    })?)
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
/// When `reset_portrait_emotion` is `true` (app startup `load_role`), portrait emotion resets to `neutral`; when switching roles it is `false` to preserve each role's last portrait state.
pub async fn load_role_impl(
    state: &AppState,
    role_id: &str,
    reset_portrait_emotion: bool,
) -> Result<RoleData, CommandError> {
    let role = state
        .storage
        .load_role(role_id)
        ?;
    let role = Arc::new(role);

    state.directory_plugins.set_active_role_id(role_id);
    state
        .directory_plugins
        .ensure_role_plugin_state(role_id, role.plugin_state_ui_baseline());

    state.invalidate_personality_cache_for_role(role_id);

    state
        .db_manager
        .ensure_role_runtime(role_id)
        .await
        ?;

    if reset_portrait_emotion {
        state
            .db_manager
            .set_current_emotion(role_id, "neutral")
            .await
            ?;
    }

    let personality = state
        .get_current_personality(role_id, role.as_ref())
        .await
        ?;

    let current_scene = state
        .db_manager
        .get_current_scene(role_id)
        .await
        ?;
    let rt = role_runtime_extras(state, role_id, current_scene.as_deref(), role.as_ref()).await?;
    maybe_seed_initial_favorability_with_extras(state, role_id, role.as_ref(), &rt).await?;
    let current_favorability = current_favorability_for_effective_identity(
        state,
        role_id,
        rt.current_user_relation.as_str(),
    )
    .await?;

    let memory_count = state
        .memory_repo
        .count_memories(role_id)
        .await
        ?;

    let event_count = state
        .db_manager
        .count_events(role_id)
        .await
        ?;
    let session_ns = session_namespace(role_id, None);
    let effective_ollama_model =
        crate::domain::effective_llm_model::resolve_effective_ollama_model(
            state,
            role.as_ref(),
            session_ns.as_str(),
        )
        .await?;
    let relation_state =
        resolve_relation_state_for_ui(state, role_id, rt.current_user_relation.as_str()).await?;
    let remote_life_enabled = state
        .db_manager
        .get_remote_life_enabled(role_id)
        .await
        ?;
    let remote_life_pack_default = role
        .remote_presence
        .as_ref()
        .and_then(|r| r.default_enabled);

    let virtual_time_ms = state
        .db_manager
        .get_virtual_time_ms(role_id)
        .await
        ?
        .unwrap_or(0);
    let interaction =
        resolve_interaction_ui_snapshot(state, role_id, role.as_ref(), virtual_time_ms).await?;

    state
        .role_cache
        .write()
        .insert(role_id.to_string(), Arc::clone(&role));
    let plugin_backends_session_override =
        plugin_backends_override_from_slot_session(state, role.as_ref(), session_ns.as_str());
    let plugin_backends_effective = state
        .effective_plugin_backends_for_session(role.as_ref(), session_ns.as_str());
    let plugin_backends_effective_sources =
        state.effective_plugin_backend_sources_for_session(role.as_ref(), session_ns.as_str());
    let (slot_registry_pack, slot_registry_effective, slot_session_overridden_keys) =
        slot_registry_role_info_fields(state, role.as_ref(), session_ns.as_str());

    Ok(RoleData {
        role_id: role_id.to_string(),
        name: role.name.clone(),
        version: role.version.clone(),
        author: role.author.clone(),
        description: role.description.clone(),
        personality_vector: personality.to_vec7(),
        current_favorability,
        current_emotion: state
            .db_manager
            .get_current_emotion(role_id)
            .await
            ?
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
        blueprint_groups_pack: blueprint_groups_pack(role.as_ref()),
    })
}
/// Ensure manifest `role_id` has `role_runtime` (auto [`load_role_impl`] when missing).
pub(crate) async fn ensure_manifest_role_ready(
    state: &AppState,
    role_id: &str,
) -> Result<(), CommandError> {
    ensure_role_info_ready(state, role_id, None).await
}

/// Ensure manifest or session namespace has `role_runtime` before building [`RoleInfo`].
async fn ensure_role_info_ready(
    state: &AppState,
    role_id: &str,
    session_id: Option<&str>,
) -> Result<(), CommandError> {
    let session_ns = session_namespace(role_id, session_id);
    if state
        .db_manager
        .role_runtime_exists(session_ns.as_str())
        .await?
    {
        return Ok(());
    }
    if session_id.map(str::trim).filter(|s| !s.is_empty()).is_some() {
        state
            .db_manager
            .ensure_role_runtime(session_ns.as_str())
            .await?;
        return Ok(());
    }
    load_role_impl(state, role_id, false).await?;
    Ok(())
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
pub async fn get_role_info_impl(
    state: &AppState,
    role_id: &str,
    session_id: Option<&str>,
) -> Result<RoleInfo, CommandError> {
    ensure_role_info_ready(state, role_id, session_id).await?;
    let session_ns = session_namespace(role_id, session_id);

    let role = state
        .load_role_cached_async(role_id)
        .await
        ?;
    let plugin_backends_session_override =
        plugin_backends_override_from_slot_session(state, role.as_ref(), session_ns.as_str());
    let plugin_backends_effective = state
        .effective_plugin_backends_for_session(role.as_ref(), session_ns.as_str());
    let plugin_backends_effective_sources =
        state.effective_plugin_backend_sources_for_session(role.as_ref(), session_ns.as_str());

    let current_scene = state
        .db_manager
        .get_current_scene(role_id)
        .await
        ?;
    let user_presence_scene = state
        .db_manager
        .get_user_presence_scene(role_id)
        .await
        ?;
    let rt = role_runtime_extras(state, role_id, current_scene.as_deref(), role.as_ref()).await?;
    maybe_seed_initial_favorability_with_extras(state, role_id, role.as_ref(), &rt).await?;

    let personality = state
        .get_current_personality(role_id, role.as_ref())
        .await
        ?;

    let last_interaction = state
        .db_manager
        .get_latest_memory_created_at(role_id)
        .await
        ?
        .map(|t| t.to_rfc3339());

    let scenes = state
        .storage
        .list_scene_ids(role_id)
        ?;
    let scene_labels: Vec<SceneLabelEntry> = scenes
        .iter()
        .map(|id| SceneLabelEntry {
            id: id.clone(),
            label: state.storage.scene_display_name_for_role(role.as_ref(), id),
        })
        .collect();
    let virtual_time_ms = state
        .db_manager
        .get_virtual_time_ms(role_id)
        .await
        ?
        .unwrap_or(0);
    let current_favorability = current_favorability_for_effective_identity(
        state,
        role_id,
        rt.current_user_relation.as_str(),
    )
    .await?;
    let effective_ollama_model =
        crate::domain::effective_llm_model::resolve_effective_ollama_model(
            state,
            role.as_ref(),
            session_ns.as_str(),
        )
        .await?;
    let relation_state =
        resolve_relation_state_for_ui(state, role_id, rt.current_user_relation.as_str()).await?;
    let remote_life_enabled = state
        .db_manager
        .get_remote_life_enabled(role_id)
        .await
        ?;
    let remote_life_pack_default = role
        .remote_presence
        .as_ref()
        .and_then(|r| r.default_enabled);

    let interaction =
        resolve_interaction_ui_snapshot(state, role_id, role.as_ref(), virtual_time_ms).await?;

    let (knowledge_enabled, knowledge_chunk_count) = match &role.knowledge_index {
        Some(idx) => (true, idx.chunks.len() as i32),
        None => (false, 0),
    };

    let (slot_registry_pack, slot_registry_effective, slot_session_overridden_keys) =
        slot_registry_role_info_fields(state, role.as_ref(), session_ns.as_str());

    Ok(RoleInfo {
        role_id: role_id.to_string(),
        role_name: role.name.clone(),
        version: role.version.clone(),
        author: role.author.clone(),
        description: role.description.clone(),
        current_favorability,
        current_emotion: state
            .db_manager
            .get_current_emotion(role_id)
            .await
            ?
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
        blueprint_groups_pack: blueprint_groups_pack(role.as_ref()),
        dual_core_enabled: role.dual_core_gated(),
        pipeline_experimental_actions: role
            .pipeline_experimental
            .as_ref()
            .map(|steps| steps.iter().map(|s| s.action.clone()).collect())
            .unwrap_or_default(),
    })
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
pub async fn list_roles_impl(state: &AppState) -> Result<Vec<RoleSummary>, CommandError> {
    let list_dev = crate::env_flags::list_dev_roles_enabled();
    let roles = state
        .storage
        .load_all_roles()
        ?;
    Ok(roles
        .into_iter()
        .filter(|r| list_dev || !r.dev_only)
        .map(|r| RoleSummary {
            id: r.id,
            name: r.name,
            version: r.version,
            author: r.author,
        })
        .collect())
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
pub async fn switch_role_impl(state: &AppState, role_id: &str) -> Result<RoleInfo, CommandError> {
    load_role_impl(state, role_id, false).await?;
    get_role_info_impl(state, role_id, None).await
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn load_role(
    role_id: String,
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<RoleData, CommandError> {
    if let Some(conn) = app.try_state::<crate::kernel_lifecycle::SharedKernelConnection>() {
        crate::kernel_attach::KernelHttpClient::load_role_via_http(&conn, role_id.trim())
            .await?;
    }
    load_role_impl(&state, &role_id, true).await
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn get_role_info(
    req: GetRoleInfoRequest,
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<RoleInfo, CommandError> {
    if let Some(conn) = app.try_state::<crate::kernel_lifecycle::SharedKernelConnection>() {
        match crate::kernel_attach::KernelHttpClient::get_role_info_via_http(&conn, &req).await {
            Ok(info) => return Ok(info),
            Err(AppError::RoleRuntimeNotReady) => {
                crate::kernel_attach::KernelHttpClient::load_role_via_http(
                    &conn,
                    req.role_id.trim(),
                )
                .await?;
                return crate::kernel_attach::KernelHttpClient::get_role_info_via_http(&conn, &req)
                    .await
                    .map_err(Into::into);
            }
            Err(e) => return Err(e.into()),
        }
    }
    get_role_info_impl(&state, &req.role_id, req.session_id.as_deref()).await
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn list_roles(state: State<'_, SharedAppState>) -> Result<Vec<RoleSummary>, CommandError> {
    list_roles_impl(&state).await
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub async fn switch_role(
    role_id: String,
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<RoleInfo, CommandError> {
    if let Some(conn) = app.try_state::<crate::kernel_lifecycle::SharedKernelConnection>() {
        crate::kernel_attach::KernelHttpClient::load_role_via_http(&conn, role_id.trim())
            .await?;
        let req = GetRoleInfoRequest {
            role_id: role_id.clone(),
            session_id: None,
        };
        return crate::kernel_attach::KernelHttpClient::get_role_info_via_http(&conn, &req)
            .await
            .map_err(Into::into);
    }
    switch_role_impl(&state, &role_id).await
}

/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
/// Deletes the local role directory and DB state for that manifest role (including `__sess__` session namespaces).
pub async fn delete_role_impl(state: &AppState, role_id: String) -> Result<Value, CommandError> {
    let rid = role_id.trim();
    if rid.is_empty() {
        return Err(AppError::InvalidParameter("role_id required".into()).into());
    }
    let removed_ns = state
        .db_manager
        .delete_all_data_for_manifest_role(rid)
        .await
        ?;
    let chat_location = state
        .load_role_cached_async(rid)
        .await
        .ok()
        .map(|r| r.pack_chat_storage_config.location.clone());
    let mirror_root = crate::infrastructure::chat_storage::resolve_role_chat_storage_root(
        state.directory_plugins.app_data_dir(),
        state.storage.roles_dir(),
        rid,
        chat_location.as_deref(),
    );
    if let Err(e) =
        crate::infrastructure::chat_storage::delete_mirror_tree_for_role(&mirror_root, rid).await
    {
        tracing::warn!(
            target: "oclive_chat_storage",
            role_id = %rid,
            error = %e,
            "delete_mirror_tree_for_role failed"
        );
    }
    for ns in &removed_ns {
        state.clear_all_session_slot_overrides(ns);
    }
    let dir = state.storage.roles_dir().join(rid);
    if dir.exists() {
        let dir_owned = dir.clone();
        tokio::task::spawn_blocking(move || std::fs::remove_dir_all(&dir_owned))
            .await
            .map_err(|e| format!("delete_role: join {e}"))?
            .map_err(|e: std::io::Error| e.to_string())?;
    }
    state.directory_plugins.remove_role_plugin_state(rid)?;
    state.role_cache.write().remove(rid);
    state.invalidate_personality_cache_for_role(rid);
    Ok(json!({ "ok": true, "role_id": rid }))
}

/// Strips the Windows verbatim path prefix `\\?\` to avoid frontend path issues.
fn path_string_for_frontend(p: &std::path::Path) -> String {
    let s = p.to_string_lossy();
    const VERBATIM: &str = "\\\\?\\";
    if let Some(stripped) = s.strip_prefix(VERBATIM) {
        stripped.to_string()
    } else {
        s.into_owned()
    }
}

/// Reads role pack asset bytes (`roles/{role_id}/{relative}`); returns `None` when the file does not exist.
///
/// # Errors
///
/// Returns a string when the file exists but cannot be read from disk.
#[tauri::command]
pub fn read_role_asset_bytes(
    role_id: String,
    relative: String,
    state: State<'_, SharedAppState>,
) -> Result<Option<Vec<u8>>, CommandError> {
    let p = state.storage.role_asset_path(&role_id, &relative);
    if !p.is_file() {
        return Ok(None);
    }
    Ok(Some(std::fs::read(&p)?))
}

/// Resolves the absolute path for `roles/{role_id}/{relative}`; when the file exists, the frontend can load it via `convertFileSrc`.
#[tauri::command]
#[must_use]
pub fn resolve_role_asset_path(
    role_id: String,
    relative: String,
    state: State<'_, SharedAppState>,
) -> Option<String> {
    let p = state.storage.role_asset_path(&role_id, &relative);
    if p.is_file() {
        return Some(path_string_for_frontend(&p));
    }
    None
}
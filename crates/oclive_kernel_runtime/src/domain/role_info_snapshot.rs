//! `get_role_info` 运行时快照：与 Tauri `get_role_info_impl` 同源，供 OOCP / HTTP 复用。

use crate::domain::chat_engine::conversation_state_role_id;
use crate::domain::life_schedule::resolve_life_state;
use crate::domain::user_identity::resolve_effective_user_relation_key;
use crate::error::{AppError, Result};
use crate::models::dto::{LifeStateDto, RoleData, RoleInfo, SceneLabelEntry, UserRelationDto};
use crate::models::{InteractionMode, Role};
use crate::state::KernelAppState;

fn user_relation_display_label(id: &str, name: &str) -> String {
    let t = name.trim();
    if !t.is_empty() && t != id {
        return name.to_string();
    }
    match id {
        "classmate" => "同学".to_string(),
        "friend" => "好友".to_string(),
        "family" => "家人".to_string(),
        "sibling" | "siblings" => "兄弟姐妹".to_string(),
        "parent" | "parents" => "父母".to_string(),
        "lover" => "恋人".to_string(),
        "rival" => "较劲".to_string(),
        "guardian" => "监护人".to_string(),
        "partner" => "伴侣".to_string(),
        "cousin" => "表亲".to_string(),
        "relative" => "亲戚".to_string(),
        "stranger" => "陌生人".to_string(),
        "teacher" => "老师".to_string(),
        "colleague" => "同事".to_string(),
        _ => id.to_string(),
    }
}

fn user_relations_to_dto(role: &Role) -> Vec<UserRelationDto> {
    role.user_relations
        .iter()
        .map(|r| UserRelationDto {
            id: r.id.clone(),
            name: user_relation_display_label(&r.id, &r.name),
            prompt_hint: r.prompt_hint.clone(),
            favor_multiplier: r.favor_multiplier,
            initial_favorability: r.initial_favorability_clamped(),
        })
        .collect()
}

pub(crate) struct RoleRuntimeExtras {
    pub user_relations: Vec<UserRelationDto>,
    pub default_relation: String,
    pub current_user_relation: String,
    pub use_manifest_default: bool,
    pub event_impact_factor: f64,
}

async fn effective_event_impact(state: &KernelAppState, role_id: &str, role: &Role) -> Result<f64> {
    Ok(state
        .db_manager
        .get_event_impact_factor(role_id)
        .await?
        .unwrap_or(role.evolution_config.event_impact_factor))
}

async fn effective_user_relation(
    state: &KernelAppState,
    role_id: &str,
    scene_id: Option<&str>,
    role: &Role,
) -> Result<String> {
    resolve_effective_user_relation_key(state, role, role_id, scene_id).await
}

pub(crate) async fn role_runtime_extras(
    state: &KernelAppState,
    role_id: &str,
    scene_id: Option<&str>,
    role: &Role,
) -> Result<RoleRuntimeExtras> {
    let use_manifest_default = state.db_manager.get_use_manifest_default(role_id).await?;
    Ok(RoleRuntimeExtras {
        user_relations: user_relations_to_dto(role),
        default_relation: role.default_relation.clone(),
        current_user_relation: effective_user_relation(state, role_id, scene_id, role).await?,
        use_manifest_default,
        event_impact_factor: effective_event_impact(state, role_id, role).await?,
    })
}

pub(crate) async fn maybe_seed_initial_favorability_with_extras(
    state: &KernelAppState,
    role_id: &str,
    role: &Role,
    rt: &RoleRuntimeExtras,
) -> Result<()> {
    let memory_count = state.memory_repo.count_memories(role_id).await?;
    let eff = rt.current_user_relation.as_str();
    let seed = role.initial_favorability_for_relation(eff);
    state
        .db_manager
        .ensure_identity_stats_row(role_id, eff, seed)
        .await?;
    let fav = state
        .db_manager
        .get_favorability_for_identity(role_id, eff)
        .await?
        .unwrap_or(0.0);
    if memory_count > 0 || fav != 0.0 {
        return Ok(());
    }
    state
        .db_manager
        .set_identity_favorability_value(role_id, eff, seed)
        .await?;
    Ok(())
}

pub(crate) async fn current_favorability_for_effective_identity(
    state: &KernelAppState,
    role_id: &str,
    effective_relation_key: &str,
) -> Result<f64> {
    state
        .db_manager
        .favorability_for_identity_with_runtime_fallback(role_id, effective_relation_key)
        .await
}

pub(crate) async fn resolve_relation_state_for_ui(
    state: &KernelAppState,
    role_id: &str,
    effective_relation_key: &str,
) -> Result<String> {
    let mut relation_state = state
        .db_manager
        .get_relation_state_for_identity(role_id, effective_relation_key)
        .await?;
    if relation_state.is_none() {
        relation_state = state.db_manager.get_relation_state(role_id).await?;
    }
    Ok(relation_state.unwrap_or_else(|| "Stranger".to_string()))
}

struct InteractionUiSnapshot {
    mode_str: String,
    pack_default: Option<String>,
    current_life: Option<LifeStateDto>,
}

async fn resolve_interaction_ui_snapshot(
    state: &KernelAppState,
    role_id: &str,
    role: &Role,
    virtual_time_ms: i64,
) -> Result<InteractionUiSnapshot> {
    state
        .db_manager
        .ensure_interaction_mode_seeded(role_id, role.interaction_mode.as_deref())
        .await?;
    let mode = state.db_manager.get_interaction_mode(role_id).await?;
    let mode_str = mode.as_str().to_string();
    let pack_default = InteractionMode::pack_default_for_api(role.interaction_mode.as_deref());
    let current_life = if mode.is_immersive() {
        role.life_schedule
            .as_ref()
            .and_then(|s| resolve_life_state(virtual_time_ms, s))
            .map(|st| LifeStateDto::from(&st))
    } else {
        None
    };
    Ok(InteractionUiSnapshot {
        mode_str,
        pack_default,
        current_life,
    })
}

/// 与桌面 `get_role_info` 命令返回的 `RoleInfo` 字段一致。
pub async fn get_role_info_snapshot(
    state: &KernelAppState,
    role_id: &str,
    session_id: Option<&str>,
) -> Result<RoleInfo> {
    let session_ns = conversation_state_role_id(role_id, session_id);
    if !state
        .db_manager
        .role_runtime_exists(session_ns.as_str())
        .await?
    {
        return Err(AppError::InvalidParameter(
            "Role runtime not initialized; call load_role first".to_string(),
        ));
    }

    let role = state.load_role_cached(role_id)?;
    let plugin_backends_session_override = state.session_backend_override(session_ns.as_str());
    let plugin_backends_effective =
        state.effective_plugin_backends_for_session(role.as_ref(), session_ns.as_str());
    let plugin_backends_effective_sources =
        state.effective_plugin_backend_sources_for_session(session_ns.as_str());

    let current_scene = state.db_manager.get_current_scene(role_id).await?;
    let user_presence_scene = state.db_manager.get_user_presence_scene(role_id).await?;
    let rt = role_runtime_extras(state, role_id, current_scene.as_deref(), role.as_ref()).await?;
    maybe_seed_initial_favorability_with_extras(state, role_id, role.as_ref(), &rt).await?;

    let personality = state
        .get_current_personality(role_id, role.as_ref())
        .await?;

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
            label: state.storage.scene_display_name(role_id, id),
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
    let effective_ollama_model = role.resolve_ollama_model(state.global_chat_model().as_str());
    let relation_state =
        resolve_relation_state_for_ui(state, role_id, rt.current_user_relation.as_str()).await?;
    let remote_life_enabled = state.db_manager.get_remote_life_enabled(role_id).await?;
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
        plugin_backends: role.plugin_backends.clone(),
        plugin_backends_session_override,
        plugin_backends_effective,
        plugin_backends_effective_sources,
        knowledge_enabled,
        knowledge_chunk_count,
        pack_ui_config: role.ui_config.clone(),
        pack_ui_baseline: role.plugin_state_ui_baseline().clone(),
        author_pack: role.author_pack.clone(),
    })
}

/// `load_role` 在磁盘加载与 Tauri 侧副作用（目录插件、`ensure_role_runtime`、可选立绘重置）完成之后的数据装配。
pub async fn build_role_data(
    state: &KernelAppState,
    role_id: &str,
    role: &Role,
) -> Result<RoleData> {
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
    let effective_ollama_model = role.resolve_ollama_model(state.global_chat_model().as_str());
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

    let session_ns = conversation_state_role_id(role_id, None);
    let plugin_backends_session_override = state.session_backend_override(session_ns.as_str());
    let plugin_backends_effective =
        state.effective_plugin_backends_for_session(role, session_ns.as_str());
    let plugin_backends_effective_sources =
        state.effective_plugin_backend_sources_for_session(session_ns.as_str());

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
        plugin_backends: role.plugin_backends.clone(),
        plugin_backends_session_override,
        plugin_backends_effective,
        plugin_backends_effective_sources,
        pack_ui_config: role.ui_config.clone(),
        pack_ui_baseline: role.plugin_state_ui_baseline().clone(),
        author_pack: role.author_pack.clone(),
    })
}

#[cfg(test)]
mod display_label_tests {
    use super::user_relation_display_label;

    #[test]
    fn prefers_custom_name_when_differs_from_id() {
        assert_eq!(user_relation_display_label("friend", "死党"), "死党");
    }

    #[test]
    fn fallback_when_name_equals_id() {
        assert_eq!(
            user_relation_display_label("classmate", "classmate"),
            "同学"
        );
        assert_eq!(
            user_relation_display_label("sibling", "sibling"),
            "兄弟姐妹"
        );
        assert_eq!(user_relation_display_label("parent", "parent"), "父母");
    }
}

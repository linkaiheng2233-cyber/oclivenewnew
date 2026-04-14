//! WEEK3-004：load_role / get_role_info / query_memories / query_events / create_event
//!
//! **Manual check（场景切换后亲密度）**：`per_scene` 角色为两场景绑定不同身份并分别写入好感后，仅切换场景不调聊天；
//! 预期 UI / `get_role_info` 的 `current_favorability` 与当前场景对应身份在 `role_identity_stats` 中的值一致，
//! 而非停留在全局列上一场景镜像值。

use oclivenewnew_tauri::api::event::{create_event_impl, query_events_impl};
use oclivenewnew_tauri::api::export::export_chat_logs_impl;
use oclivenewnew_tauri::api::memory::query_memories_impl;
use oclivenewnew_tauri::api::role::{
    get_plugin_resolution_debug_impl, get_role_info_impl, list_roles_impl, load_role_impl,
    set_evolution_factor_impl,
    set_scene_user_relation_impl, set_session_plugin_backend_impl, set_user_relation_impl,
    switch_role_impl,
};
use oclivenewnew_tauri::api::scene::switch_scene_impl;
use oclivenewnew_tauri::domain::chat_engine::process_message;
use oclivenewnew_tauri::infrastructure::MockLlmClient;
use oclivenewnew_tauri::models::dto::{
    CreateEventRequest, ExportChatLogsRequest, GetPluginResolutionDebugRequest, QueryEventsRequest,
    QueryMemoriesRequest, SendMessageRequest, SetEvolutionFactorRequest,
    SetSceneUserRelationRequest, SetSessionPluginBackendRequest, SetUserRelationRequest,
    SwitchSceneRequest,
};
use oclivenewnew_tauri::models::{
    role::IdentityBinding, MemoryBackend, PersonalitySource, PluginBackendSource,
};
use oclivenewnew_tauri::models::dto::{API_VERSION, SCHEMA_VERSION};
use oclivenewnew_tauri::state::AppState;
use std::path::PathBuf;
use std::sync::Arc;

fn roles_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../roles")
}

#[tokio::test]
async fn week3_004_load_role_and_get_info() {
    let llm = Arc::new(MockLlmClient {
        reply: "ok".to_string(),
    });
    let state = AppState::new_in_memory_with_llm(llm, roles_dir())
        .await
        .expect("state");

    let data = load_role_impl(&state, "mumu", true)
        .await
        .expect("load_role");
    assert_eq!(data.role_id, "mumu");
    assert_eq!(data.identity_binding, IdentityBinding::Global);
    assert_eq!(data.personality_vector.len(), 7);
    assert_eq!(
        data.current_emotion.to_ascii_lowercase(),
        "neutral",
        "启动 load_role 立绘应回到正常"
    );
    assert_eq!(
        data.personality_source,
        PersonalitySource::Vector,
        "mumu 包未写 personality_source 时应默认为 vector"
    );

    let info = get_role_info_impl(&state, "mumu", None)
        .await
        .expect("get_role_info");
    assert_eq!(info.role_id, "mumu");
    assert_eq!(info.current_emotion.to_ascii_lowercase(), "neutral");
    assert_eq!(
        info.personality_source,
        PersonalitySource::Vector,
        "get_role_info 应与包内 evolution.personality_source 一致"
    );
}

#[tokio::test]
async fn week3_004_get_role_info_before_runtime_fails() {
    let llm = Arc::new(MockLlmClient {
        reply: "ok".to_string(),
    });
    let state = AppState::new_in_memory_with_llm(llm, roles_dir())
        .await
        .expect("state");

    let err = get_role_info_impl(&state, "mumu", None).await.unwrap_err();
    assert!(err.contains("load_role"));
}

#[tokio::test]
async fn week3_004_session_backend_override_uses_session_namespace() {
    let llm = Arc::new(MockLlmClient {
        reply: "ok".to_string(),
    });
    let state = AppState::new_in_memory_with_llm(llm, roles_dir())
        .await
        .expect("state");
    load_role_impl(&state, "mumu", true)
        .await
        .expect("load_role");

    let session_info = set_session_plugin_backend_impl(
        &state,
        &SetSessionPluginBackendRequest {
            role_id: "mumu".to_string(),
            module: "memory".to_string(),
            backend: Some(Some("remote".to_string())),
            local_memory_provider_id: None,
            session_id: Some("sess-a".to_string()),
        },
    )
    .await
    .expect("set session backend");
    assert_eq!(session_info.plugin_backends_effective.memory, MemoryBackend::Remote);
    assert_eq!(
        session_info.plugin_backends_effective_sources.memory,
        PluginBackendSource::SessionOverride
    );

    let same_session = get_role_info_impl(&state, "mumu", Some("sess-a"))
        .await
        .expect("get same session role info");
    assert_eq!(same_session.plugin_backends_effective.memory, MemoryBackend::Remote);
    assert_eq!(
        same_session.plugin_backends_effective_sources.memory,
        PluginBackendSource::SessionOverride
    );

    let default_session = get_role_info_impl(&state, "mumu", None)
        .await
        .expect("get default session role info");
    assert_eq!(
        default_session.plugin_backends_effective.memory,
        default_session.plugin_backends.memory
    );
    assert_eq!(
        default_session.plugin_backends_effective_sources.memory,
        PluginBackendSource::PackDefault
    );
}

#[tokio::test]
async fn week3_004_session_memory_local_provider_id_without_touching_memory_enum() {
    let llm = Arc::new(MockLlmClient {
        reply: "ok".to_string(),
    });
    let state = AppState::new_in_memory_with_llm(llm, roles_dir())
        .await
        .expect("state");
    load_role_impl(&state, "mumu", true)
        .await
        .expect("load_role");

    set_session_plugin_backend_impl(
        &state,
        &SetSessionPluginBackendRequest {
            role_id: "mumu".to_string(),
            module: "memory".to_string(),
            backend: Some(Some("local".to_string())),
            local_memory_provider_id: None,
            session_id: Some("sess-local-pick".to_string()),
        },
    )
    .await
    .expect("set memory local");

    let after_pick = set_session_plugin_backend_impl(
        &state,
        &SetSessionPluginBackendRequest {
            role_id: "mumu".to_string(),
            module: "memory".to_string(),
            backend: None,
            local_memory_provider_id: Some("  my_local_mem  ".to_string()),
            session_id: Some("sess-local-pick".to_string()),
        },
    )
    .await
    .expect("set local memory provider id");

    assert_eq!(
        after_pick.plugin_backends_effective.memory,
        MemoryBackend::Local
    );
    assert_eq!(
        after_pick
            .plugin_backends_effective
            .local_memory_provider_id
            .as_deref(),
        Some("my_local_mem")
    );
    assert_eq!(
        after_pick.plugin_backends_effective_sources.memory,
        PluginBackendSource::SessionOverride
    );

    let cleared = set_session_plugin_backend_impl(
        &state,
        &SetSessionPluginBackendRequest {
            role_id: "mumu".to_string(),
            module: "memory".to_string(),
            backend: None,
            local_memory_provider_id: Some("   ".to_string()),
            session_id: Some("sess-local-pick".to_string()),
        },
    )
    .await
    .expect("clear local provider id override");

    assert_eq!(
        cleared.plugin_backends_effective.memory,
        MemoryBackend::Local
    );
    assert!(cleared.plugin_backends_effective.local_memory_provider_id.is_none());
    assert_eq!(
        cleared.plugin_backends_effective_sources.memory,
        PluginBackendSource::SessionOverride
    );
}

#[tokio::test]
async fn week3_004_session_backend_explicit_null_clears_module_override() {
    let llm = Arc::new(MockLlmClient {
        reply: "ok".to_string(),
    });
    let state = AppState::new_in_memory_with_llm(llm, roles_dir())
        .await
        .expect("state");
    load_role_impl(&state, "mumu", true)
        .await
        .expect("load_role");

    let set_remote = set_session_plugin_backend_impl(
        &state,
        &SetSessionPluginBackendRequest {
            role_id: "mumu".to_string(),
            module: "memory".to_string(),
            backend: Some(Some("remote".to_string())),
            local_memory_provider_id: None,
            session_id: Some("sess-clear-memory".to_string()),
        },
    )
    .await
    .expect("set remote override");
    assert_eq!(set_remote.plugin_backends_effective.memory, MemoryBackend::Remote);
    assert_eq!(
        set_remote.plugin_backends_effective_sources.memory,
        PluginBackendSource::SessionOverride
    );

    let cleared = set_session_plugin_backend_impl(
        &state,
        &SetSessionPluginBackendRequest {
            role_id: "mumu".to_string(),
            module: "memory".to_string(),
            backend: Some(None),
            local_memory_provider_id: None,
            session_id: Some("sess-clear-memory".to_string()),
        },
    )
    .await
    .expect("clear memory override");

    assert_eq!(
        cleared.plugin_backends_effective.memory,
        cleared.plugin_backends.memory
    );
    assert_eq!(
        cleared.plugin_backends_effective_sources.memory,
        PluginBackendSource::PackDefault
    );
}

#[tokio::test]
async fn week3_004_local_memory_provider_id_rejects_non_memory_module() {
    let llm = Arc::new(MockLlmClient {
        reply: "ok".to_string(),
    });
    let state = AppState::new_in_memory_with_llm(llm, roles_dir())
        .await
        .expect("state");
    load_role_impl(&state, "mumu", true)
        .await
        .expect("load_role");

    let err = set_session_plugin_backend_impl(
        &state,
        &SetSessionPluginBackendRequest {
            role_id: "mumu".to_string(),
            module: "prompt".to_string(),
            backend: None,
            local_memory_provider_id: Some("provider_a".to_string()),
            session_id: Some("sess-invalid-provider-field".to_string()),
        },
    )
    .await
    .unwrap_err();
    assert!(err.contains("module=memory"));
}

#[tokio::test]
async fn week3_004_plugin_resolution_debug_reports_session_override_and_namespace() {
    let llm = Arc::new(MockLlmClient {
        reply: "ok".to_string(),
    });
    let state = AppState::new_in_memory_with_llm(llm, roles_dir())
        .await
        .expect("state");
    load_role_impl(&state, "mumu", true)
        .await
        .expect("load_role");

    set_session_plugin_backend_impl(
        &state,
        &SetSessionPluginBackendRequest {
            role_id: "mumu".to_string(),
            module: "memory".to_string(),
            backend: Some(Some("remote".to_string())),
            local_memory_provider_id: None,
            session_id: Some("diag-sess".to_string()),
        },
    )
    .await
    .expect("set override");

    let debug = get_plugin_resolution_debug_impl(
        &state,
        &GetPluginResolutionDebugRequest {
            role_id: "mumu".to_string(),
            session_id: Some("diag-sess".to_string()),
        },
    )
    .await
    .expect("debug info");

    assert_eq!(debug.session_namespace, "mumu__sess__diag-sess");
    assert_eq!(debug.api_version, API_VERSION);
    assert_eq!(debug.schema_version, SCHEMA_VERSION);
    assert!(!debug.app_version.trim().is_empty());
    assert_eq!(debug.local_provider_count, 0);
    assert!(debug.local_provider_ids.is_empty());
    assert_eq!(debug.plugin_backends_effective.memory, MemoryBackend::Remote);
    assert_eq!(
        debug.plugin_backends_effective_sources.memory,
        PluginBackendSource::SessionOverride
    );
}

#[tokio::test]
async fn week3_004_export_chat_logs_with_plugin_debug_includes_section() {
    let llm = Arc::new(MockLlmClient {
        reply: "ok".to_string(),
    });
    let state = AppState::new_in_memory_with_llm(llm, roles_dir())
        .await
        .expect("state");
    load_role_impl(&state, "mumu", true)
        .await
        .expect("load_role");

    let out = export_chat_logs_impl(
        &state,
        &ExportChatLogsRequest {
            role_id: Some("mumu".to_string()),
            all_roles: false,
            format: "txt".to_string(),
            include_plugin_resolution_debug: true,
            session_id: None,
        },
    )
    .await
    .expect("export");
    assert!(out.content.contains("## 插件解析诊断"));
    assert!(out.content.contains("app_version:"));
    assert!(out.content.contains("session_namespace: mumu"));
    assert!(out.content.contains("local_providers: count=0 ids=none"));
}

#[tokio::test]
async fn week3_004_query_memories_and_events() {
    let llm = Arc::new(MockLlmClient {
        reply: "模拟".to_string(),
    });
    let state = AppState::new_in_memory_with_llm(llm, roles_dir())
        .await
        .expect("state");

    let req = SendMessageRequest {
        role_id: "mumu".to_string(),
        user_message: "hi".to_string(),
        scene_id: None,
        session_id: None,
    };
    process_message(&state, &req).await.expect("send");

    let mems = query_memories_impl(
        &state,
        &QueryMemoriesRequest {
            role_id: "mumu".to_string(),
            limit: 10,
            offset: 0,
        },
    )
    .await
    .expect("query_memories");
    assert!(!mems.is_empty());

    let evs = query_events_impl(
        &state,
        &QueryEventsRequest {
            role_id: "mumu".to_string(),
            limit: 10,
            offset: 0,
        },
    )
    .await
    .expect("query_events");
    assert!(!evs.is_empty());
    assert!(evs[0].user_emotion.is_some());
    assert!(evs[0].bot_emotion.is_some());
}

#[tokio::test]
async fn week3_004_create_event_and_query() {
    let llm = Arc::new(MockLlmClient {
        reply: "ok".to_string(),
    });
    let state = AppState::new_in_memory_with_llm(llm, roles_dir())
        .await
        .expect("state");

    load_role_impl(&state, "mumu", true)
        .await
        .expect("load_role");

    let created = create_event_impl(
        &state,
        &CreateEventRequest {
            role_id: "mumu".to_string(),
            event_type: "Praise".to_string(),
            description: Some("manual".to_string()),
        },
    )
    .await
    .expect("create_event");
    assert_eq!(created.event_type, "Praise");

    let list = query_events_impl(
        &state,
        &QueryEventsRequest {
            role_id: "mumu".to_string(),
            limit: 10,
            offset: 0,
        },
    )
    .await
    .expect("query");
    assert!(list.iter().any(|e| {
        e.event_type == created.event_type && e.description.as_deref() == Some("manual")
    }));
}

#[tokio::test]
async fn week3_004_create_event_invalid_type() {
    let llm = Arc::new(MockLlmClient {
        reply: "ok".to_string(),
    });
    let state = AppState::new_in_memory_with_llm(llm, roles_dir())
        .await
        .expect("state");

    load_role_impl(&state, "mumu", true)
        .await
        .expect("load_role");

    let err = create_event_impl(
        &state,
        &CreateEventRequest {
            role_id: "mumu".to_string(),
            event_type: "InvalidType".to_string(),
            description: None,
        },
    )
    .await
    .unwrap_err();
    assert!(err.contains("[INVALID_PARAMETER]"));
    assert!(err.contains("Invalid event_type"));
}

#[tokio::test]
async fn week3_004_query_limits_return_invalid_parameter_code() {
    let llm = Arc::new(MockLlmClient {
        reply: "ok".to_string(),
    });
    let state = AppState::new_in_memory_with_llm(llm, roles_dir())
        .await
        .expect("state");

    let err = query_memories_impl(
        &state,
        &QueryMemoriesRequest {
            role_id: "mumu".to_string(),
            limit: 0,
            offset: 0,
        },
    )
    .await
    .unwrap_err();
    assert!(err.contains("[INVALID_PARAMETER]"));

    let err2 = query_events_impl(
        &state,
        &QueryEventsRequest {
            role_id: "mumu".to_string(),
            limit: 0,
            offset: 0,
        },
    )
    .await
    .unwrap_err();
    assert!(err2.contains("[INVALID_PARAMETER]"));
}

#[tokio::test]
async fn week3_004_list_roles_and_switch_role() {
    let llm = Arc::new(MockLlmClient {
        reply: "ok".to_string(),
    });
    let state = AppState::new_in_memory_with_llm(llm, roles_dir())
        .await
        .expect("state");

    let roles = list_roles_impl(&state).await.expect("list_roles");
    assert!(!roles.is_empty());
    assert!(roles.iter().any(|r| r.id == "mumu"));

    let switched = switch_role_impl(&state, "mumu").await.expect("switch_role");
    assert_eq!(switched.role_id, "mumu");
}

#[tokio::test]
async fn week3_004_set_user_relation_and_evolution_factor() {
    let llm = Arc::new(MockLlmClient {
        reply: "ok".to_string(),
    });
    let state = AppState::new_in_memory_with_llm(llm, roles_dir())
        .await
        .expect("state");
    load_role_impl(&state, "mumu", true)
        .await
        .expect("load_role");

    let info = set_user_relation_impl(
        &state,
        &SetUserRelationRequest {
            role_id: "mumu".to_string(),
            relation: "family".to_string(),
        },
    )
    .await
    .expect("set_user_relation");
    assert_eq!(info.current_user_relation, "family");

    let info2 = set_evolution_factor_impl(
        &state,
        &SetEvolutionFactorRequest {
            role_id: "mumu".to_string(),
            event_impact_factor: 1.5,
        },
    )
    .await
    .expect("set_evolution_factor");
    assert!((info2.event_impact_factor - 1.5_f64).abs() < 1e-9);

    let err = set_evolution_factor_impl(
        &state,
        &SetEvolutionFactorRequest {
            role_id: "mumu".to_string(),
            event_impact_factor: 0.01,
        },
    )
    .await
    .expect_err("factor too low");
    assert!(err.contains("INVALID_PARAMETER") || err.contains("event_impact"));

    let err2 = set_user_relation_impl(
        &state,
        &SetUserRelationRequest {
            role_id: "mumu".to_string(),
            relation: "not_a_relation".to_string(),
        },
    )
    .await
    .expect_err("bad relation");
    assert!(err2.contains("unknown relation"));
}

#[tokio::test]
async fn week3_004_set_scene_user_relation_validates_and_persists() {
    let llm = Arc::new(MockLlmClient {
        reply: "ok".to_string(),
    });
    let state = AppState::new_in_memory_with_llm(llm, roles_dir())
        .await
        .expect("state");
    load_role_impl(&state, "shimeng", true)
        .await
        .expect("load_role");

    let info = set_scene_user_relation_impl(
        &state,
        &SetSceneUserRelationRequest {
            role_id: "shimeng".to_string(),
            scene_id: "default".to_string(),
            relation: "parent".to_string(),
        },
    )
    .await
    .expect("set_scene_user_relation");
    assert_eq!(info.role_id, "shimeng");

    let scene_relation = state
        .db_manager
        .get_user_relation_for_scene("shimeng", "default")
        .await
        .expect("read scene relation");
    assert_eq!(scene_relation.as_deref(), Some("parent"));

    let bad_scene = set_scene_user_relation_impl(
        &state,
        &SetSceneUserRelationRequest {
            role_id: "shimeng".to_string(),
            scene_id: "not_exist".to_string(),
            relation: "parent".to_string(),
        },
    )
    .await
    .expect_err("invalid scene");
    assert!(bad_scene.contains("scene_id not in role pack"));
}

#[tokio::test]
async fn week3_004_scene_relation_overrides_global_in_chat() {
    let llm = Arc::new(MockLlmClient {
        reply: "ok".to_string(),
    });
    let state = AppState::new_in_memory_with_llm(llm, roles_dir())
        .await
        .expect("state");
    load_role_impl(&state, "shimeng", true)
        .await
        .expect("load_role");

    set_user_relation_impl(
        &state,
        &SetUserRelationRequest {
            role_id: "shimeng".to_string(),
            relation: "classmate".to_string(),
        },
    )
    .await
    .expect("set global relation");
    set_scene_user_relation_impl(
        &state,
        &SetSceneUserRelationRequest {
            role_id: "shimeng".to_string(),
            scene_id: "default".to_string(),
            relation: "parent".to_string(),
        },
    )
    .await
    .expect("set scene relation");

    let in_default = process_message(
        &state,
        &SendMessageRequest {
            role_id: "shimeng".to_string(),
            user_message: "你今天怎么样".to_string(),
            scene_id: Some("default".to_string()),
            session_id: None,
        },
    )
    .await
    .expect("send in default");
    let in_school = process_message(
        &state,
        &SendMessageRequest {
            role_id: "shimeng".to_string(),
            user_message: "你今天怎么样".to_string(),
            scene_id: Some("school".to_string()),
            session_id: None,
        },
    )
    .await
    .expect("send in school");

    assert!(
        in_default.favorability_delta.abs() >= in_school.favorability_delta.abs(),
        "default scene relation should apply parent multiplier >= global classmate"
    );
}

/// `get_role_info` / `switch_scene` 返回的亲密度应按**当前场景解析出的身份**从 `role_identity_stats` 读取；
/// 全局 `role_runtime.current_favorability` 可能仍是上一身份镜像，不得单独作为 UI 来源。
#[tokio::test]
async fn week3_004_get_role_info_favor_follows_scene_identity_not_global_column() {
    let llm = Arc::new(MockLlmClient {
        reply: "ok".to_string(),
    });
    let state = AppState::new_in_memory_with_llm(llm, roles_dir())
        .await
        .expect("state");

    let rid = "shimeng";
    load_role_impl(&state, rid, true).await.expect("load_role");

    set_scene_user_relation_impl(
        &state,
        &SetSceneUserRelationRequest {
            role_id: rid.to_string(),
            scene_id: "default".to_string(),
            relation: "parent".to_string(),
        },
    )
    .await
    .expect("set default -> parent");
    set_scene_user_relation_impl(
        &state,
        &SetSceneUserRelationRequest {
            role_id: rid.to_string(),
            scene_id: "school".to_string(),
            relation: "classmate".to_string(),
        },
    )
    .await
    .expect("set school -> classmate");

    switch_scene_impl(
        &state,
        &SwitchSceneRequest {
            role_id: rid.to_string(),
            scene_id: "default".to_string(),
            together: true,
        },
    )
    .await
    .expect("switch default");

    state
        .db_manager
        .set_identity_favorability_value(rid, "parent", 10.0)
        .await
        .expect("parent favor 10");

    switch_scene_impl(
        &state,
        &SwitchSceneRequest {
            role_id: rid.to_string(),
            scene_id: "school".to_string(),
            together: true,
        },
    )
    .await
    .expect("switch school");

    state
        .db_manager
        .set_identity_favorability_value(rid, "classmate", 90.0)
        .await
        .expect("classmate favor 90 mirrors global to 90");

    let back_default = switch_scene_impl(
        &state,
        &SwitchSceneRequest {
            role_id: rid.to_string(),
            scene_id: "default".to_string(),
            together: true,
        },
    )
    .await
    .expect("switch back default");

    assert!(
        (back_default.role.current_favorability - 10.0).abs() < 1e-6,
        "expected parent identity favor 10, got {} (global column is stale 90)",
        back_default.role.current_favorability
    );
    assert_eq!(back_default.role.current_user_relation, "parent");
}

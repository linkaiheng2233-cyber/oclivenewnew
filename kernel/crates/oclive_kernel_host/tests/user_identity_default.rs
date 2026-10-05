#![allow(clippy::expect_used, clippy::unwrap_used)]

use async_trait::async_trait;
use oclive_kernel_contracts::LlmClient;
use oclive_kernel_host::domain::host_profile::load_host_profile_file;
use oclive_kernel_host::domain::user_identity_loader::resolve_active_user_identity;
use oclive_kernel_host::service::role::{
    get_user_identity_state_impl, set_scene_user_identity_impl, set_user_identity_impl,
};
use oclive_kernel_host::state::{AppState, AppStateBuilder};
use oclive_kernel_types::models::dto::{
    GetUserIdentityStateRequest, SetSceneUserIdentityRequest, SetUserIdentityRequest,
    UserIdentityStateResponse, OCLIVE_DEFAULT_IDENTITY_SENTINEL,
};
use oclive_kernel_types::models::role::{IdentityBinding, Role, UserRelation};
use oclive_kernel_types::models::role_manifest_disk::disk_manifest_from_role;
use serde_json::json;
use std::sync::Arc;

struct NoModel;

#[async_trait]
impl LlmClient for NoModel {
    async fn generate(&self, _model: &str, _prompt: &str) -> oclive_kernel_types::Result<String> {
        panic!("identity selection must not generate")
    }

    async fn generate_tag(
        &self,
        _model: &str,
        _prompt: &str,
    ) -> oclive_kernel_types::Result<String> {
        panic!("identity selection must not classify")
    }
}

struct Fixture {
    root: tempfile::TempDir,
    state: AppState,
}

impl Fixture {
    async fn new(per_scene: bool, host_default: Option<&str>) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let roles = dir.path().join("roles");
        let role_dir = roles.join("identity-test");
        let identities = role_dir.join("user_identities");
        std::fs::create_dir_all(&identities).unwrap();
        let role = Role {
            id: "identity-test".into(),
            name: "Identity fixture".into(),
            author: "test".into(),
            version: "1.0.0".into(),
            core_personality: "Synthetic identity fixture".into(),
            default_relation: "relation-pack".into(),
            identity_binding: if per_scene {
                IdentityBinding::PerScene
            } else {
                IdentityBinding::Global
            },
            user_relations: ["host", "pack", "choice"]
                .into_iter()
                .map(|id| UserRelation {
                    id: format!("relation-{id}"),
                    name: id.into(),
                    prompt_hint: format!("legacy-{id}"),
                    favor_multiplier: 1.0,
                    initial_favorability: 20.0,
                })
                .collect(),
            ..Default::default()
        };
        let mut disk_manifest = disk_manifest_from_role(&role);
        disk_manifest.identity_binding = role.identity_binding;
        let mut manifest = serde_json::to_value(disk_manifest).unwrap();
        manifest["scenes"] = json!(["default", "other"]);
        std::fs::write(role_dir.join("manifest.json"), manifest.to_string()).unwrap();
        let entries = ["host", "pack", "choice"]
            .into_iter()
            .map(|id| {
                std::fs::write(
                    identities.join(format!("{id}.md")),
                    format!("template-{id}"),
                )
                .unwrap();
                (
                    id.to_string(),
                    json!({"display_name":id,"template_file":format!("{id}.md"),
                        "maps_to_relation_id":format!("relation-{id}")}),
                )
            })
            .collect::<serde_json::Map<String, serde_json::Value>>();
        std::fs::write(
            identities.join("index.json"),
            json!({"schema_version":1,"default_identity_id":"pack","identities":entries})
                .to_string(),
        )
        .unwrap();
        let profile_path = dir.path().join("distro.oclive.toml");
        let profile_text = match host_default {
            Some(id) => format!(
                "distro_id = \"chat-pro\"\n[user_identity]\ndefault_id = {id:?}\nallowed_ids = [\"choice\"]\n"
            ),
            None => "distro_id = \"chat-pro\"\n[user_identity]\nallowed_ids = [\"choice\"]\n".into(),
        };
        std::fs::write(&profile_path, profile_text).unwrap();
        let profile = load_host_profile_file(&profile_path).unwrap();
        assert_eq!(profile.distro_id, "chat-pro");
        assert_eq!(profile.user_identity.default_id.as_deref(), host_default);
        let state = AppStateBuilder::in_memory_test(Arc::new(NoModel), &roles, None)
            .with_app_data_dir(dir.path().join("app-data"))
            .with_host_profile(profile)
            .build()
            .await
            .unwrap();
        assert_eq!(
            state
                .storage
                .load_role("identity-test")
                .unwrap()
                .identity_binding,
            role.identity_binding
        );
        Self { root: dir, state }
    }

    async fn view(&self, scene: Option<&str>) -> UserIdentityStateResponse {
        get_user_identity_state_impl(
            &self.state,
            &GetUserIdentityStateRequest {
                role_id: "identity-test".into(),
                scene_id: scene.map(str::to_string),
            },
        )
        .await
        .unwrap()
    }

    async fn assert_identity(&self, scene: Option<&str>, id: &str, following_default: bool) {
        let view = self.view(scene).await;
        assert_eq!(view.current_identity_id, id);
        assert_eq!(view.use_manifest_default, following_default);
        assert_eq!(view.effective_relation_key, format!("relation-{id}"));
        let role = self.state.storage.load_role("identity-test").unwrap();
        let resolved = resolve_active_user_identity(&self.state, &role, "identity-test", scene)
            .await
            .unwrap();
        assert_eq!(resolved.identity_id, id);
        assert_eq!(resolved.template_body, format!("template-{id}"));
        assert_eq!(resolved.relation_key, view.effective_relation_key);
    }

    async fn close(self) {
        self.state.directory_plugins.shutdown_all();
        self.state.db_manager.close_pool().await;
        drop(self.state);
        self.root.close().unwrap();
    }
}

#[tokio::test]
async fn global_initial_default_uses_the_distro_and_keeps_the_compatibility_flag() {
    let fixture = Fixture::new(false, Some("host")).await;
    fixture.assert_identity(None, "host", true).await;
    assert_eq!(fixture.view(None).await.default_identity_id, "host");
    assert_eq!(
        fixture
            .state
            .db_manager
            .get_global_identity_state("identity-test")
            .await
            .unwrap(),
        (true, None)
    );
    fixture.close().await;
}

#[tokio::test]
async fn global_explicit_choice_wins_and_restore_default_resynchronizes_the_relation() {
    let fixture = Fixture::new(false, Some("host")).await;
    fixture.view(None).await;
    let pick = SetUserIdentityRequest {
        role_id: "identity-test".into(),
        identity_id: "choice".into(),
    };
    let selected = set_user_identity_impl(&fixture.state, &pick).await.unwrap();
    assert_eq!(selected.default_identity_id, "host");
    fixture.assert_identity(None, "choice", false).await;
    let restored = set_user_identity_impl(
        &fixture.state,
        &SetUserIdentityRequest {
            role_id: pick.role_id,
            identity_id: OCLIVE_DEFAULT_IDENTITY_SENTINEL.into(),
        },
    )
    .await
    .unwrap();
    assert_eq!(restored.current_identity_id, "host");
    fixture.assert_identity(None, "host", true).await;
    assert_eq!(
        fixture
            .state
            .db_manager
            .get_user_relation("identity-test")
            .await
            .unwrap()
            .as_deref(),
        Some("relation-host")
    );
    fixture.close().await;
}

#[tokio::test]
async fn per_scene_without_a_pick_does_not_inherit_a_global_explicit_pick_or_flag() {
    let fixture = Fixture::new(true, Some("host")).await;
    fixture.view(Some("default")).await;
    fixture
        .state
        .db_manager
        .set_use_manifest_default_identity("identity-test", false)
        .await
        .unwrap();
    fixture
        .state
        .db_manager
        .set_active_user_identity_id("identity-test", "choice")
        .await
        .unwrap();
    for scene in [None, Some("default"), Some("other")] {
        fixture.assert_identity(scene, "host", true).await;
        assert_eq!(fixture.view(scene).await.default_identity_id, "host");
    }
    fixture.close().await;
}

#[tokio::test]
async fn per_scene_explicit_choice_stays_local_and_restore_follows_the_distro() {
    let fixture = Fixture::new(true, Some("host")).await;
    fixture.view(Some("default")).await;
    let pick = SetSceneUserIdentityRequest {
        role_id: "identity-test".into(),
        scene_id: "default".into(),
        identity_id: "choice".into(),
    };
    set_scene_user_identity_impl(&fixture.state, &pick)
        .await
        .unwrap();
    fixture
        .assert_identity(Some("default"), "choice", false)
        .await;
    fixture.assert_identity(Some("other"), "host", true).await;
    set_scene_user_identity_impl(
        &fixture.state,
        &SetSceneUserIdentityRequest {
            identity_id: OCLIVE_DEFAULT_IDENTITY_SENTINEL.into(),
            ..pick
        },
    )
    .await
    .unwrap();
    fixture.assert_identity(Some("default"), "host", true).await;
    assert_eq!(
        fixture
            .state
            .db_manager
            .get_user_identity_id_for_scene("identity-test", "default")
            .await
            .unwrap(),
        None
    );
    fixture.close().await;
}

#[tokio::test]
async fn absent_or_unknown_distro_default_falls_back_to_the_pack_for_both_binding_modes() {
    for per_scene in [false, true] {
        for configured in [None, Some("unknown")] {
            let fixture = Fixture::new(per_scene, configured).await;
            let scene = per_scene.then_some("default");
            fixture.assert_identity(scene, "pack", true).await;
            assert_eq!(fixture.view(scene).await.default_identity_id, "pack");
            fixture.close().await;
        }
    }
}

#[tokio::test]
async fn explicit_selection_keeps_unknown_id_and_host_allow_list_rejections() {
    for per_scene in [false, true] {
        let fixture = Fixture::new(per_scene, Some("host")).await;
        let scene = per_scene.then_some("default");
        fixture.view(scene).await;
        for id in ["unknown", "host"] {
            let error = if per_scene {
                set_scene_user_identity_impl(
                    &fixture.state,
                    &SetSceneUserIdentityRequest {
                        role_id: "identity-test".into(),
                        scene_id: "default".into(),
                        identity_id: id.into(),
                    },
                )
                .await
                .unwrap_err()
            } else {
                set_user_identity_impl(
                    &fixture.state,
                    &SetUserIdentityRequest {
                        role_id: "identity-test".into(),
                        identity_id: id.into(),
                    },
                )
                .await
                .unwrap_err()
            };
            let message = error.to_string();
            assert!(
                message.contains(if id == "unknown" {
                    "unknown user identity"
                } else {
                    "not allowed by host"
                }),
                "{message}"
            );
        }
        fixture.assert_identity(scene, "host", true).await;
        fixture.close().await;
    }
}

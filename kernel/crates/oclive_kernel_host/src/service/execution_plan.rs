//! Read-only capability registry and execution-plan diagnostics.

use oclive_kernel_types::{
    AppError, ExecutionPlanDiagnostics, GetExecutionPlanDiagnosticsRequest, Role,
    EXECUTION_PLAN_DIAGNOSTIC_SCHEMA_VERSION,
};

use crate::command_error::CommandError;
use crate::domain::execution_plan::{compile_execution_plan, CompileExecutionPlanInput};
use crate::infrastructure::capability_registry::build_capability_registry;
use crate::state::AppState;

#[must_use]
pub fn build_execution_plan_diagnostics_for_role(
    state: &AppState,
    role: &Role,
    session_id: Option<&str>,
) -> ExecutionPlanDiagnostics {
    let session_namespace = crate::service::role::session_namespace(&role.id, session_id);
    let core_backends =
        state.effective_plugin_backends_for_session(role, session_namespace.as_str());
    let registry = build_capability_registry(
        state.directory_plugins.as_ref(),
        state.high_risk_grants.as_ref(),
        state.host_profile.as_ref(),
        &role.id,
    );
    let plan = compile_execution_plan(&CompileExecutionPlanInput {
        role_id: &role.id,
        distro_id: &state.host_profile.distro_id,
        core_backends: core_backends.as_ref(),
        extensions: &role.blueprint_extensions,
        registry: &registry,
    });
    ExecutionPlanDiagnostics {
        schema_version: EXECUTION_PLAN_DIAGNOSTIC_SCHEMA_VERSION,
        plan,
        capability_registry: registry,
    }
}

/// Reject role activation when one or more required blueprint capabilities
/// cannot be resolved. Optional extension degradation remains non-fatal.
///
/// # Errors
///
/// Returns [`AppError::InvalidParameter`] with stable diagnostic codes when the
/// plan contains blocked required extensions.
pub fn ensure_role_execution_plan_activatable(
    state: &AppState,
    role: &Role,
) -> Result<(), AppError> {
    let diagnostics = build_execution_plan_diagnostics_for_role(state, role, None);
    if diagnostics.plan.activatable {
        return Ok(());
    }
    let blocked = diagnostics
        .plan
        .extensions
        .iter()
        .filter(|extension| !extension.active && extension.required)
        .map(|extension| {
            format!(
                "{} [{}]",
                extension.instance_id,
                extension.reason_codes.join(",")
            )
        })
        .collect::<Vec<_>>();
    Err(AppError::InvalidParameter(format!(
        "execution_plan: required blueprint capabilities unavailable for role {}: {}",
        role.id,
        blocked.join("; ")
    )))
}

/// Load role metadata without activating it, then compile an on-demand plan
/// snapshot. This remains available even when required extensions are blocked.
///
/// # Errors
///
/// Returns role-pack load or request validation failures.
pub async fn get_execution_plan_diagnostics_impl(
    state: &AppState,
    request: &GetExecutionPlanDiagnosticsRequest,
) -> Result<ExecutionPlanDiagnostics, CommandError> {
    let role_id = request.role_id.trim();
    if role_id.is_empty() {
        return Err(AppError::InvalidParameter("role_id is required".into()).into());
    }
    let storage = state.storage.clone();
    let role_id_owned = role_id.to_string();
    let role = tokio::task::spawn_blocking(move || storage.load_role(&role_id_owned))
        .await
        .map_err(|error| {
            AppError::Unknown(format!("execution plan role load task failed: {error}"))
        })??;
    let resource_diagnostics = state.resource_coordinator.refresh().await;
    let mut diagnostics =
        build_execution_plan_diagnostics_for_role(state, &role, request.session_id.as_deref());
    diagnostics.plan.resource_coordination = resource_diagnostics.state;
    diagnostics.plan.resource_plan = Some(resource_diagnostics.candidate_plan);
    Ok(diagnostics)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::host_profile::HostProfile;
    use crate::infrastructure::MockLlmClient;
    use crate::service::role::load_role_impl;
    use crate::state::AppStateBuilder;
    use std::fs;
    use std::path::Path;
    use std::sync::Arc;
    use tempfile::tempdir;

    fn write_v4_pack(role_dir: &Path, required: bool, capability: &str, provider: Option<&str>) {
        let extension_id = "com.example.extension";
        let extension_dir = role_dir.join("blueprint/extensions").join(extension_id);
        fs::create_dir_all(&extension_dir).unwrap();
        fs::write(extension_dir.join("config.json"), r#"{"enabled":true}"#).unwrap();
        fs::write(
            role_dir.join(oclive_validation::PIPELINE_BLUEPRINT_FILENAME),
            serde_json::json!({
                "schema_version": 4,
                "meta": {
                    "id": role_dir.file_name().unwrap().to_string_lossy(),
                    "name": "Plan test",
                    "version": "1.0.0",
                    "author": "test",
                    "description": "test",
                    "relations": {
                        "friend": {
                            "initial_favorability": 50,
                            "favor_multiplier": 1
                        }
                    },
                    "default_relation": "friend"
                },
                "slot_registry": {
                    "llm": {
                        "type": "llm",
                        "label": "LLM",
                        "backend": "ollama",
                        "position": 0
                    }
                },
                "extensions": {
                    extension_id: {
                        "capability": capability,
                        "provider": provider,
                        "required": required,
                        "config_schema_version": 1,
                        "config_ref": format!(
                            "blueprint/extensions/{extension_id}/config.json"
                        )
                    }
                }
            })
            .to_string(),
        )
        .unwrap();
    }

    #[tokio::test]
    async fn blocked_required_extension_still_has_read_only_diagnostics() {
        let dir = tempdir().unwrap();
        let roles = dir.path().join("roles");
        let role_dir = roles.join("blocked");
        write_v4_pack(&role_dir, true, "render.live2d", None);
        let state = AppStateBuilder::in_memory_test(
            Arc::new(MockLlmClient { reply: "ok".into() }),
            &roles,
            None,
        )
        .with_host_profile(HostProfile {
            distro_id: "desktop".into(),
            ..HostProfile::default()
        })
        .build()
        .await
        .unwrap();

        let report = get_execution_plan_diagnostics_impl(
            &state,
            &GetExecutionPlanDiagnosticsRequest {
                role_id: "blocked".into(),
                session_id: None,
            },
        )
        .await
        .unwrap();
        assert!(!report.plan.activatable);
        assert_eq!(
            report.plan.extensions[0].reason_codes,
            vec!["capability_consumer_unavailable"]
        );
        let resource_plan = report.plan.resource_plan.as_ref().expect("resource plan");
        assert_ne!(resource_plan.compiled_from_revision, 0);
        assert_eq!(
            resource_plan.state,
            oclive_kernel_types::ResourceCandidatePlanState::Ready
        );

        let error = load_role_impl(&state, "blocked", false)
            .await
            .unwrap_err()
            .to_string();
        assert!(error.contains("execution_plan:"), "{error}");
    }

    #[tokio::test]
    async fn registered_consumer_and_ready_provider_allow_activation() {
        let dir = tempdir().unwrap();
        let roles = dir.path().join("distros/chat-pro/roles");
        let role_dir = roles.join("voice-role");
        write_v4_pack(&role_dir, true, "voice.asr", Some("com.example.voice"));
        let plugin_dir = dir
            .path()
            .join("distros/chat-pro/plugins/com.example.voice");
        fs::create_dir_all(&plugin_dir).unwrap();
        fs::write(
            plugin_dir.join("manifest.json"),
            serde_json::json!({
                "schema_version": 1,
                "id": "com.example.voice",
                "version": "1.0.0",
                "provides": ["voice.asr"],
                "process": {"command": "not-started-by-plan"}
            })
            .to_string(),
        )
        .unwrap();
        let state = AppStateBuilder::in_memory_test(
            Arc::new(MockLlmClient { reply: "ok".into() }),
            &roles,
            None,
        )
        .with_host_profile(HostProfile {
            distro_id: "desktop".into(),
            ..HostProfile::default()
        })
        .build()
        .await
        .unwrap();

        let report = get_execution_plan_diagnostics_impl(
            &state,
            &GetExecutionPlanDiagnosticsRequest {
                role_id: "voice-role".into(),
                session_id: None,
            },
        )
        .await
        .unwrap();
        assert!(report.plan.activatable);
        assert!(report.plan.extensions[0].active);
        assert_eq!(
            report.plan.extensions[0].selected_provider_id.as_deref(),
            Some("com.example.voice")
        );
        load_role_impl(&state, "voice-role", false).await.unwrap();
    }

    #[tokio::test]
    async fn same_role_reports_distro_specific_activation_without_mutating_pack() {
        let dir = tempdir().unwrap();
        let roles = dir.path().join("distros/chat-pro/roles");
        let role_dir = roles.join("portable-voice");
        write_v4_pack(&role_dir, true, "voice.asr", Some("com.example.voice"));
        let plugin_dir = dir
            .path()
            .join("distros/chat-pro/plugins/com.example.voice");
        fs::create_dir_all(&plugin_dir).unwrap();
        fs::write(
            plugin_dir.join("manifest.json"),
            serde_json::json!({
                "schema_version": 1,
                "id": "com.example.voice",
                "version": "1.0.0",
                "provides": ["voice.asr"],
                "process": {"command": "not-started-by-plan"}
            })
            .to_string(),
        )
        .unwrap();

        let desktop = AppStateBuilder::in_memory_test(
            Arc::new(MockLlmClient { reply: "ok".into() }),
            &roles,
            None,
        )
        .with_host_profile(HostProfile {
            distro_id: "desktop".into(),
            ..HostProfile::default()
        })
        .build()
        .await
        .unwrap();
        let vscode = AppStateBuilder::in_memory_test(
            Arc::new(MockLlmClient { reply: "ok".into() }),
            &roles,
            None,
        )
        .with_host_profile(HostProfile {
            distro_id: "vscode".into(),
            ..HostProfile::default()
        })
        .build()
        .await
        .unwrap();
        let request = GetExecutionPlanDiagnosticsRequest {
            role_id: "portable-voice".into(),
            session_id: None,
        };

        let desktop_report = get_execution_plan_diagnostics_impl(&desktop, &request)
            .await
            .unwrap();
        let vscode_report = get_execution_plan_diagnostics_impl(&vscode, &request)
            .await
            .unwrap();

        assert!(desktop_report.plan.activatable);
        assert!(!vscode_report.plan.activatable);
        assert_eq!(
            vscode_report.plan.extensions[0].reason_codes,
            vec!["capability_consumer_unavailable"]
        );
        assert!(role_dir
            .join(oclive_validation::PIPELINE_BLUEPRINT_FILENAME)
            .is_file());
    }
}

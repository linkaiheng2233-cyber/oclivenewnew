//! Pure compilation of blueprint capability intent into a read-only execution plan.
//!
//! Stable turn ordering remains owned by `process_message` / `turn_pipeline`.
//! This module resolves capability dependencies only; it does not start
//! providers, allocate resources, or execute blueprint-authored steps.

use std::collections::BTreeMap;

use oclive_kernel_types::{
    CapabilityProviderAvailability, CapabilityRegistryDiagnostic, ExecutionPlan,
    ExecutionPlanCoreNode, ExecutionPlanDiagnostic, ExecutionPlanDiagnosticSeverity,
    ExecutionPlanExtension, ExecutionPlanFlowTemplate, ExtensionPlanStatus, PluginBackends,
    ResourceCoordinationDiagnosticState, EXECUTION_PLAN_DIAGNOSTIC_SCHEMA_VERSION,
};
use oclive_validation::BlueprintExtensionDecl;

const CORE_NODE_IDS: [&str; 6] = ["memory", "emotion", "event", "prompt", "llm", "agent"];

pub struct CompileExecutionPlanInput<'a> {
    pub role_id: &'a str,
    pub distro_id: &'a str,
    pub core_backends: &'a PluginBackends,
    pub extensions: &'a BTreeMap<String, BlueprintExtensionDecl>,
    pub registry: &'a CapabilityRegistryDiagnostic,
}

fn backend_wire(value: &impl serde::Serialize) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(str::to_string))
        .unwrap_or_else(|| "unknown".to_string())
}

fn core_nodes(backends: &PluginBackends) -> Vec<ExecutionPlanCoreNode> {
    let values = [
        backend_wire(&backends.memory),
        backend_wire(&backends.emotion),
        backend_wire(&backends.event),
        backend_wire(&backends.prompt),
        backend_wire(&backends.llm),
        backend_wire(&backends.agent),
    ];
    CORE_NODE_IDS
        .into_iter()
        .zip(values)
        .map(|(node_id, backend)| ExecutionPlanCoreNode {
            node_id: node_id.to_string(),
            enabled: backend != "none",
            backend,
        })
        .collect()
}

fn provider_reason_message(code: &str, provider_id: Option<&str>, capability: &str) -> String {
    match code {
        "capability_consumer_unavailable" => {
            format!("host has no registered consumer for capability {capability}")
        }
        "requested_provider_not_installed" => format!(
            "requested provider {} is not installed",
            provider_id.unwrap_or("<unknown>")
        ),
        "requested_provider_capability_mismatch" => format!(
            "provider {} does not declare capability {capability}",
            provider_id.unwrap_or("<unknown>")
        ),
        "provider_disabled" => format!(
            "provider {} is disabled for this role",
            provider_id.unwrap_or("<unknown>")
        ),
        "provider_manifest_incompatible" => format!(
            "provider {} manifest is incompatible with host execution requirements",
            provider_id.unwrap_or("<unknown>")
        ),
        "provider_not_executable" => format!(
            "provider {} has no executable directory-plugin process",
            provider_id.unwrap_or("<unknown>")
        ),
        "provider_dependency_unavailable" => format!(
            "provider {} has missing or incompatible dependencies",
            provider_id.unwrap_or("<unknown>")
        ),
        "provider_permission_required" => format!(
            "provider {} requires user permission",
            provider_id.unwrap_or("<unknown>")
        ),
        _ => format!("no ready provider is available for capability {capability}"),
    }
}

fn unavailable_extension(
    instance_id: &str,
    declaration: &BlueprintExtensionDecl,
    candidates: Vec<String>,
    reason_codes: Vec<String>,
    provider_id: Option<&str>,
) -> (ExecutionPlanExtension, Vec<ExecutionPlanDiagnostic>) {
    let status = if declaration.required {
        ExtensionPlanStatus::Blocked
    } else {
        ExtensionPlanStatus::Degraded
    };
    let severity = if declaration.required {
        ExecutionPlanDiagnosticSeverity::Error
    } else {
        ExecutionPlanDiagnosticSeverity::Warning
    };
    let diagnostic_codes = if reason_codes.is_empty() {
        vec!["no_provider_available".to_string()]
    } else {
        reason_codes
    };
    let diagnostics = diagnostic_codes
        .iter()
        .map(|code| ExecutionPlanDiagnostic {
            code: code.clone(),
            severity,
            message: provider_reason_message(code, provider_id, &declaration.capability),
            instance_id: Some(instance_id.to_string()),
            provider_id: provider_id.map(str::to_string),
            suggested_provider_id: declaration.provider.clone().filter(|requested| {
                code == "requested_provider_not_installed" && requested == provider_id.unwrap_or("")
            }),
        })
        .collect();
    (
        ExecutionPlanExtension {
            instance_id: instance_id.to_string(),
            capability: declaration.capability.clone(),
            required: declaration.required,
            config_schema_version: declaration.config_schema_version,
            config_ref: declaration.config_ref.clone(),
            requested_provider_id: declaration.provider.clone(),
            selected_provider_id: None,
            selected_provider_version: None,
            status,
            active: false,
            provider_candidates: candidates,
            reason_codes: diagnostic_codes,
        },
        diagnostics,
    )
}

#[must_use]
pub fn compile_execution_plan(input: &CompileExecutionPlanInput<'_>) -> ExecutionPlan {
    let consumers: BTreeMap<&str, _> = input
        .registry
        .consumers
        .iter()
        .map(|consumer| (consumer.capability.as_str(), consumer))
        .collect();
    let providers: BTreeMap<&str, _> = input
        .registry
        .providers
        .iter()
        .map(|provider| (provider.provider_id.as_str(), provider))
        .collect();

    let mut extensions = Vec::with_capacity(input.extensions.len());
    let mut diagnostics = Vec::new();

    for (instance_id, declaration) in input.extensions {
        if !consumers.contains_key(declaration.capability.as_str()) {
            let (entry, mut entry_diagnostics) = unavailable_extension(
                instance_id,
                declaration,
                Vec::new(),
                vec!["capability_consumer_unavailable".to_string()],
                declaration.provider.as_deref(),
            );
            extensions.push(entry);
            diagnostics.append(&mut entry_diagnostics);
            continue;
        }

        let mut candidates: Vec<_> = input
            .registry
            .providers
            .iter()
            .filter(|provider| {
                provider
                    .provides
                    .iter()
                    .any(|capability| capability == &declaration.capability)
            })
            .collect();
        candidates.sort_by(|left, right| left.provider_id.cmp(&right.provider_id));
        let candidate_ids = candidates
            .iter()
            .map(|provider| provider.provider_id.clone())
            .collect::<Vec<_>>();

        let selected = if let Some(requested) = declaration.provider.as_deref() {
            let Some(provider) = providers.get(requested).copied() else {
                let (entry, mut entry_diagnostics) = unavailable_extension(
                    instance_id,
                    declaration,
                    candidate_ids,
                    vec!["requested_provider_not_installed".to_string()],
                    Some(requested),
                );
                extensions.push(entry);
                diagnostics.append(&mut entry_diagnostics);
                continue;
            };
            if !provider
                .provides
                .iter()
                .any(|capability| capability == &declaration.capability)
            {
                let (entry, mut entry_diagnostics) = unavailable_extension(
                    instance_id,
                    declaration,
                    candidate_ids,
                    vec!["requested_provider_capability_mismatch".to_string()],
                    Some(requested),
                );
                extensions.push(entry);
                diagnostics.append(&mut entry_diagnostics);
                continue;
            }
            Some(provider)
        } else {
            candidates
                .iter()
                .find(|provider| provider.availability == CapabilityProviderAvailability::Ready)
                .copied()
        };

        let Some(selected) = selected else {
            let mut reason_codes = candidates
                .iter()
                .flat_map(|provider| provider.reason_codes.iter().cloned())
                .collect::<Vec<_>>();
            reason_codes.sort();
            reason_codes.dedup();
            let (entry, mut entry_diagnostics) =
                unavailable_extension(instance_id, declaration, candidate_ids, reason_codes, None);
            extensions.push(entry);
            diagnostics.append(&mut entry_diagnostics);
            continue;
        };

        if selected.availability != CapabilityProviderAvailability::Ready {
            let (entry, mut entry_diagnostics) = unavailable_extension(
                instance_id,
                declaration,
                candidate_ids,
                selected.reason_codes.clone(),
                Some(selected.provider_id.as_str()),
            );
            extensions.push(entry);
            diagnostics.append(&mut entry_diagnostics);
            continue;
        }

        extensions.push(ExecutionPlanExtension {
            instance_id: instance_id.clone(),
            capability: declaration.capability.clone(),
            required: declaration.required,
            config_schema_version: declaration.config_schema_version,
            config_ref: declaration.config_ref.clone(),
            requested_provider_id: declaration.provider.clone(),
            selected_provider_id: Some(selected.provider_id.clone()),
            selected_provider_version: Some(selected.version.clone()),
            status: ExtensionPlanStatus::Ready,
            active: true,
            provider_candidates: candidate_ids,
            reason_codes: Vec::new(),
        });
    }

    let activatable = extensions
        .iter()
        .all(|extension| extension.status != ExtensionPlanStatus::Blocked);
    ExecutionPlan {
        schema_version: EXECUTION_PLAN_DIAGNOSTIC_SCHEMA_VERSION,
        role_id: input.role_id.to_string(),
        distro_id: input.distro_id.to_string(),
        flow_template: ExecutionPlanFlowTemplate::CoPresentStable,
        core_nodes: core_nodes(input.core_backends),
        core_backends: input.core_backends.clone(),
        extensions,
        activatable,
        resource_coordination: ResourceCoordinationDiagnosticState::NotEvaluated,
        resource_plan: None,
        diagnostics,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oclive_kernel_types::{
        CapabilityConsumerDiagnostic, CapabilityConsumerKind, CapabilityProviderDiagnostic,
        CapabilityProviderSource, CapabilityRegistryDiagnostic,
    };

    fn declaration(required: bool, provider: Option<&str>) -> BlueprintExtensionDecl {
        BlueprintExtensionDecl {
            capability: "render.live2d".into(),
            provider: provider.map(str::to_string),
            required,
            config_schema_version: 1,
            config_ref: "blueprint/extensions/com.example.live/config.json".into(),
        }
    }

    fn registry(providers: Vec<CapabilityProviderDiagnostic>) -> CapabilityRegistryDiagnostic {
        CapabilityRegistryDiagnostic {
            schema_version: EXECUTION_PLAN_DIAGNOSTIC_SCHEMA_VERSION,
            distro_id: "desktop".into(),
            consumers: vec![CapabilityConsumerDiagnostic {
                capability: "render.live2d".into(),
                kind: CapabilityConsumerKind::SideChannel,
                consumer_id: "render.live2d".into(),
            }],
            providers,
        }
    }

    fn provider(
        id: &str,
        availability: CapabilityProviderAvailability,
    ) -> CapabilityProviderDiagnostic {
        CapabilityProviderDiagnostic {
            provider_id: id.into(),
            version: "1.2.3".into(),
            manifest_schema_version: 1,
            source: CapabilityProviderSource::Directory,
            provides: vec!["render.live2d".into()],
            availability,
            permissions: Vec::new(),
            dependency_issues: Vec::new(),
            reason_codes: if availability == CapabilityProviderAvailability::Ready {
                Vec::new()
            } else {
                vec!["provider_permission_required".into()]
            },
        }
    }

    fn compile(
        declaration: BlueprintExtensionDecl,
        registry: &CapabilityRegistryDiagnostic,
    ) -> ExecutionPlan {
        let extensions = [("com.example.live".to_string(), declaration)]
            .into_iter()
            .collect();
        compile_execution_plan(&CompileExecutionPlanInput {
            role_id: "demo",
            distro_id: "desktop",
            core_backends: &PluginBackends::default(),
            extensions: &extensions,
            registry,
        })
    }

    #[test]
    fn required_missing_provider_blocks_but_optional_degrades() {
        let registry = registry(Vec::new());
        let required = compile(declaration(true, Some("com.example.runtime")), &registry);
        assert!(!required.activatable);
        assert_eq!(required.extensions[0].status, ExtensionPlanStatus::Blocked);
        assert_eq!(
            required.diagnostics[0].code,
            "requested_provider_not_installed"
        );
        assert_eq!(
            required.diagnostics[0].suggested_provider_id.as_deref(),
            Some("com.example.runtime")
        );

        let optional = compile(declaration(false, None), &registry);
        assert!(optional.activatable);
        assert_eq!(optional.extensions[0].status, ExtensionPlanStatus::Degraded);
        assert!(!optional.extensions[0].active);
    }

    #[test]
    fn automatic_provider_selection_is_ready_then_lexicographic() {
        let registry = registry(vec![
            provider(
                "com.example.blocked",
                CapabilityProviderAvailability::PermissionRequired,
            ),
            provider("com.example.z", CapabilityProviderAvailability::Ready),
            provider("com.example.a", CapabilityProviderAvailability::Ready),
        ]);
        let plan = compile(declaration(true, None), &registry);
        assert!(plan.activatable);
        assert_eq!(
            plan.extensions[0].selected_provider_id.as_deref(),
            Some("com.example.a")
        );
    }

    #[test]
    fn explicit_provider_permission_gap_is_visible_and_blocks_required() {
        let registry = registry(vec![provider(
            "com.example.runtime",
            CapabilityProviderAvailability::PermissionRequired,
        )]);
        let plan = compile(declaration(true, Some("com.example.runtime")), &registry);
        assert!(!plan.activatable);
        assert_eq!(
            plan.extensions[0].reason_codes,
            vec!["provider_permission_required"]
        );
    }

    #[test]
    fn unregistered_consumer_never_activates_arbitrary_manifest_capability() {
        let mut registry = registry(vec![provider(
            "com.example.runtime",
            CapabilityProviderAvailability::Ready,
        )]);
        registry.consumers.clear();
        let plan = compile(declaration(false, Some("com.example.runtime")), &registry);
        assert!(plan.activatable);
        assert!(!plan.extensions[0].active);
        assert_eq!(
            plan.extensions[0].reason_codes,
            vec!["capability_consumer_unavailable"]
        );
    }

    #[test]
    fn core_plan_uses_fixed_six_node_order() {
        let plan = compile(
            declaration(false, None),
            &CapabilityRegistryDiagnostic {
                schema_version: EXECUTION_PLAN_DIAGNOSTIC_SCHEMA_VERSION,
                distro_id: "desktop".into(),
                consumers: Vec::new(),
                providers: Vec::new(),
            },
        );
        let ids = plan
            .core_nodes
            .iter()
            .map(|node| node.node_id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(ids, CORE_NODE_IDS);
        assert_eq!(
            plan.flow_template,
            ExecutionPlanFlowTemplate::CoPresentStable
        );
        assert_eq!(
            plan.resource_coordination,
            ResourceCoordinationDiagnosticState::NotEvaluated
        );
    }

    #[test]
    fn unavailable_candidate_reasons_are_deduplicated() {
        let registry = registry(vec![
            provider(
                "com.example.a",
                CapabilityProviderAvailability::PermissionRequired,
            ),
            provider(
                "com.example.b",
                CapabilityProviderAvailability::PermissionRequired,
            ),
        ]);
        let plan = compile(declaration(false, None), &registry);
        assert_eq!(
            plan.extensions[0].reason_codes,
            vec!["provider_permission_required"]
        );
        assert_eq!(plan.diagnostics.len(), 1);
    }
}

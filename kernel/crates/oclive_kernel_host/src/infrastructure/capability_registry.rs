//! Host adapter that builds a capability registry snapshot from installed
//! directory-plugin manifests, grants, role plugin state, and `HostProfile`.

use std::collections::{BTreeSet, HashMap};

use oclive_kernel_types::{
    CapabilityConsumerDiagnostic, CapabilityConsumerKind, CapabilityPermissionDiagnostic,
    CapabilityProviderAvailability, CapabilityProviderDiagnostic, CapabilityProviderSource,
    CapabilityRegistryDiagnostic, EXECUTION_PLAN_DIAGNOSTIC_SCHEMA_VERSION,
};
use oclive_validation::{manifest_declares_process_spawn, PROCESS_SPAWN};

use crate::domain::host_profile::HostProfile;
use crate::infrastructure::directory_plugins::{
    dependency_report, parse_manifest_version, DirectoryPluginRuntime,
};
use crate::infrastructure::high_risk_grants::HighRiskGrantStore;

fn registered_consumers(profile: &HostProfile) -> Vec<CapabilityConsumerDiagnostic> {
    let chat_pro_host = matches!(
        profile.distro_id.trim(),
        "" | "default" | "desktop" | "desktop-chat"
    );
    let mut consumers = Vec::new();
    if chat_pro_host {
        consumers.push(CapabilityConsumerDiagnostic {
            capability: "voice.asr".into(),
            kind: CapabilityConsumerKind::SideChannel,
            consumer_id: "chat_pro.voice".into(),
        });
    }
    consumers
}

fn provider_availability(
    disabled: bool,
    has_process: bool,
    spawn_declared: bool,
    dependency_status: &str,
    permissions: &[CapabilityPermissionDiagnostic],
) -> (CapabilityProviderAvailability, Vec<String>) {
    let mut reasons = Vec::new();
    if disabled {
        reasons.push("provider_disabled".to_string());
    }
    if has_process && !spawn_declared {
        reasons.push("provider_manifest_incompatible".to_string());
    }
    if !has_process {
        reasons.push("provider_not_executable".to_string());
    }
    if dependency_status != "ok" {
        reasons.push("provider_dependency_unavailable".to_string());
    }
    if permissions.iter().any(|permission| !permission.granted) {
        reasons.push("provider_permission_required".to_string());
    }
    reasons.sort();
    reasons.dedup();

    let availability = if reasons
        .iter()
        .any(|reason| reason == "provider_manifest_incompatible")
    {
        CapabilityProviderAvailability::ManifestIncompatible
    } else if reasons.iter().any(|reason| reason == "provider_disabled") {
        CapabilityProviderAvailability::Disabled
    } else if reasons
        .iter()
        .any(|reason| reason == "provider_not_executable")
    {
        CapabilityProviderAvailability::NotExecutable
    } else if reasons
        .iter()
        .any(|reason| reason == "provider_dependency_unavailable")
    {
        CapabilityProviderAvailability::DependencyUnavailable
    } else if reasons
        .iter()
        .any(|reason| reason == "provider_permission_required")
    {
        CapabilityProviderAvailability::PermissionRequired
    } else {
        CapabilityProviderAvailability::Ready
    };
    (availability, reasons)
}

#[must_use]
pub fn build_capability_registry(
    runtime: &DirectoryPluginRuntime,
    grants: &HighRiskGrantStore,
    profile: &HostProfile,
    role_id: &str,
) -> CapabilityRegistryDiagnostic {
    runtime.ensure_plugin_roots_scanned();
    let roots = runtime.plugin_roots.read().clone();
    let mut manifests = roots
        .iter()
        .filter_map(|(plugin_id, entry)| {
            runtime
                .load_manifest_cached(plugin_id, &entry.root)
                .ok()
                .map(|manifest| (plugin_id.clone(), manifest))
        })
        .collect::<Vec<_>>();
    manifests.sort_by(|left, right| left.0.cmp(&right.0));

    let version_by_id: HashMap<_, _> = manifests
        .iter()
        .filter_map(|(plugin_id, manifest)| {
            parse_manifest_version(&manifest.version).map(|version| (plugin_id.clone(), version))
        })
        .collect();
    let plugin_state = runtime.role_plugin_state_for(role_id);

    let providers = manifests
        .into_iter()
        .map(|(plugin_id, manifest)| {
            let mut provides = manifest
                .provides
                .iter()
                .map(|capability| capability.trim().to_string())
                .filter(|capability| !capability.is_empty())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            provides.sort();

            let has_process = manifest.process.is_some();
            let spawn_declared =
                manifest_declares_process_spawn(&manifest.permissions, has_process);
            let mut effective_permissions = manifest
                .permissions
                .iter()
                .map(|permission| permission.trim().to_string())
                .filter(|permission| !permission.is_empty())
                .collect::<BTreeSet<_>>();
            if has_process && manifest.permissions.is_empty() {
                effective_permissions.insert(PROCESS_SPAWN.to_string());
            }
            let permissions = effective_permissions
                .into_iter()
                .map(|permission| CapabilityPermissionDiagnostic {
                    granted: grants.is_permission_granted(&permission, &plugin_id),
                    permission,
                })
                .collect::<Vec<_>>();
            let (dependency_status, dependency_issues) =
                dependency_report(&manifest, &version_by_id);
            let (availability, reason_codes) = provider_availability(
                plugin_state.is_plugin_disabled(&plugin_id),
                has_process,
                spawn_declared,
                &dependency_status,
                &permissions,
            );
            CapabilityProviderDiagnostic {
                provider_id: plugin_id,
                version: manifest.version.clone(),
                manifest_schema_version: manifest.schema_version,
                source: CapabilityProviderSource::Directory,
                provides,
                availability,
                permissions,
                dependency_issues,
                reason_codes,
            }
        })
        .collect();

    CapabilityRegistryDiagnostic {
        schema_version: EXECUTION_PLAN_DIAGNOSTIC_SCHEMA_VERSION,
        distro_id: profile.distro_id.clone(),
        consumers: registered_consumers(profile),
        providers,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn write_plugin(root: &std::path::Path) {
        fs::create_dir_all(root).unwrap();
        fs::write(
            root.join("manifest.json"),
            serde_json::json!({
                "schema_version": 1,
                "id": "com.example.voice",
                "version": "1.0.0",
                "provides": ["voice.asr"],
                "permissions": ["process:spawn"],
                "process": {"command": "not-started-by-diagnostics"}
            })
            .to_string(),
        )
        .unwrap();
    }

    #[test]
    fn registry_reports_permission_gap_without_starting_provider() {
        let dir = tempdir().unwrap();
        let roles = dir.path().join("distros/chat-pro/roles");
        let plugin = dir
            .path()
            .join("distros/chat-pro/plugins/com.example.voice");
        fs::create_dir_all(&roles).unwrap();
        write_plugin(&plugin);
        let app_data = dir.path().join("app-data");
        let grants = HighRiskGrantStore::load(app_data.clone(), true);
        let profile = HostProfile {
            distro_id: "desktop".into(),
            ..HostProfile::default()
        };
        let runtime = DirectoryPluginRuntime::bootstrap_with_host_profile(
            &roles,
            &app_data,
            grants.clone(),
            profile.clone(),
            true,
        );

        let registry =
            build_capability_registry(runtime.as_ref(), grants.as_ref(), &profile, "demo");
        assert_eq!(registry.providers.len(), 1);
        assert_eq!(
            registry.providers[0].availability,
            CapabilityProviderAvailability::PermissionRequired
        );
        assert_eq!(
            registry.providers[0].permissions,
            vec![CapabilityPermissionDiagnostic {
                permission: "process:spawn".into(),
                granted: false,
            }]
        );
        assert!(registry
            .consumers
            .iter()
            .any(|consumer| consumer.capability == "voice.asr"));
    }

    #[test]
    fn registered_consumers_are_distro_specific() {
        let desktop = registered_consumers(&HostProfile {
            distro_id: "desktop".into(),
            ..HostProfile::default()
        });
        let vscode = registered_consumers(&HostProfile {
            distro_id: "vscode".into(),
            ..HostProfile::default()
        });
        assert!(desktop
            .iter()
            .any(|consumer| consumer.capability == "voice.asr"));
        assert!(vscode.is_empty());
    }
}

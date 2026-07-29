//! Resource Coordinator diagnostics and official runtime-adapter hooks.

use oclive_kernel_types::{
    ResourceAdmissionDecision, ResourceAdmissionMode, ResourceAdmissionRequest,
    ResourceAdmissionResult, ResourceControlMode, ResourceCoordinationDiagnostics,
    ResourcePriority,
};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::domain::resource_coordinator::{configured_gpu_device_index, ResourceCoordinator};
use crate::infrastructure::resource_adapters::{
    cosyvoice_reservation_mib, COSYVOICE_ADAPTER_ID, COSYVOICE_PROFILE_ID,
};
use crate::state::AppState;

const OFFICIAL_VOICE_PLUGIN_ID: &str = "com.oclive.voice.asr";
const BUNDLED_COSYVOICE_PROFILE_ID: &str = "bundled-cosyvoice2-zh";
const COSYVOICE_WORKLOAD_ID: &str = "bundled-runtime";

pub enum DirectoryPluginResourceAdmission {
    NotApplicable,
    Admitted(DirectoryPluginResourceLease, u64),
    Denied(Box<ResourceAdmissionResult>),
}

pub struct DirectoryPluginResourceLease {
    coordinator: Arc<ResourceCoordinator>,
    lease_id: String,
    release_on_drop: bool,
    _operation_guard: tokio::sync::OwnedMutexGuard<()>,
}

impl DirectoryPluginResourceLease {
    fn commit(&mut self) {
        self.release_on_drop = false;
    }
}

impl Drop for DirectoryPluginResourceLease {
    fn drop(&mut self) {
        if self.release_on_drop {
            self.coordinator.release(&self.lease_id);
        }
    }
}

pub async fn get_resource_coordination_diagnostics_impl(
    state: &AppState,
) -> ResourceCoordinationDiagnostics {
    state.resource_coordinator.refresh().await
}

fn bundled_voice_resource_request(
    plugin_id: &str,
    method: &str,
    params: &Value,
) -> Option<ResourceAdmissionRequest> {
    if plugin_id != OFFICIAL_VOICE_PLUGIN_ID || !matches!(method, "voice.warm" | "voice.speak") {
        return None;
    }
    let profile = (method == "voice.speak")
        .then(|| params.pointer("/directive/synth_profile"))
        .flatten()
        .or_else(|| params.get("profile"))
        .and_then(Value::as_str)
        .unwrap_or(BUNDLED_COSYVOICE_PROFILE_ID)
        .trim();
    if !profile.is_empty() && profile != BUNDLED_COSYVOICE_PROFILE_ID {
        return None;
    }
    let reservation_mib = cosyvoice_reservation_mib();
    Some(ResourceAdmissionRequest {
        adapter_id: COSYVOICE_ADAPTER_ID.into(),
        workload_id: COSYVOICE_WORKLOAD_ID.into(),
        profile_id: Some(COSYVOICE_PROFILE_ID.into()),
        gpu_device_index: configured_gpu_device_index(),
        reservation_mib,
        priority: if method == "voice.speak" {
            ResourcePriority::ForegroundMedia
        } else {
            ResourcePriority::BackgroundWarmup
        },
        control_mode: ResourceControlMode::Managed,
        admission_mode: ResourceAdmissionMode::Enforced,
    })
}

pub async fn prepare_directory_plugin_resource_rpc(
    state: &AppState,
    plugin_id: &str,
    method: &str,
    params: &mut Value,
) -> DirectoryPluginResourceAdmission {
    let Some(request) = bundled_voice_resource_request(plugin_id, method, params) else {
        return DirectoryPluginResourceAdmission::NotApplicable;
    };
    let operation_guard = state
        .resource_coordinator
        .lock_adapter_operation(COSYVOICE_ADAPTER_ID)
        .await;
    let reservation_mib = request.reservation_mib;
    let admission = state.resource_coordinator.admit(request).await;
    if admission.decision == ResourceAdmissionDecision::Denied {
        return DirectoryPluginResourceAdmission::Denied(Box::new(admission));
    }
    let Some(lease_id) = admission.lease.as_ref().map(|lease| lease.lease_id.clone()) else {
        return DirectoryPluginResourceAdmission::Denied(Box::new(admission));
    };
    if !params.is_object() {
        *params = json!({});
    }
    if let Some(object) = params.as_object_mut() {
        object.insert(
            "_oclive_resource_admission".into(),
            json!({
                "schema_version": 1,
                "granted": true,
                "lease_id": lease_id.clone(),
                "reservation_mib": reservation_mib,
            }),
        );
    }
    DirectoryPluginResourceAdmission::Admitted(
        DirectoryPluginResourceLease {
            coordinator: Arc::clone(&state.resource_coordinator),
            lease_id,
            release_on_drop: admission.decision != ResourceAdmissionDecision::Reused,
            _operation_guard: operation_guard,
        },
        reservation_mib,
    )
}

pub fn finalize_directory_plugin_resource_rpc(
    state: &AppState,
    admission: DirectoryPluginResourceAdmission,
    response: Option<&Value>,
) {
    let DirectoryPluginResourceAdmission::Admitted(mut lease, reservation_mib) = admission else {
        return;
    };
    let succeeded = response.and_then(Value::as_object).is_some_and(|result| {
        result.get("ok").and_then(Value::as_bool) == Some(true)
            && result.get("skipped").and_then(Value::as_bool) != Some(true)
    });
    if !succeeded {
        return;
    }
    let actual_mib = response
        .and_then(|value| value.get("load_peak_reserved_mib"))
        .and_then(Value::as_u64)
        .filter(|value| *value > 0)
        .unwrap_or(reservation_mib);
    state
        .resource_coordinator
        .activate(&lease.lease_id, Some(actual_mib));
    lease.commit();
}

pub fn release_directory_plugin_resources_for_config(
    state: &AppState,
    plugin_id: &str,
    config: &Value,
) {
    if plugin_id != OFFICIAL_VOICE_PLUGIN_ID {
        return;
    }
    let enabled = config.get("tts_expansion_enabled").and_then(Value::as_bool) == Some(true);
    let bundled = config
        .get("synth_provider")
        .and_then(Value::as_str)
        .is_none_or(|provider| provider == "bundled");
    let bundled_profile = config
        .get("tts_profile")
        .and_then(Value::as_str)
        .is_none_or(|profile| profile == BUNDLED_COSYVOICE_PROFILE_ID);
    if !enabled || !bundled || !bundled_profile {
        state
            .resource_coordinator
            .release_adapter(COSYVOICE_ADAPTER_ID);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use oclive_kernel_contracts::ResourceSnapshotSource;
    use oclive_kernel_types::{ResourceCoordinatorPolicy, ResourceSnapshot};

    struct UnavailableSnapshot;

    #[async_trait]
    impl ResourceSnapshotSource for UnavailableSnapshot {
        async fn snapshot(&self) -> ResourceSnapshot {
            ResourceSnapshot::unavailable("test", "unavailable")
        }
    }

    #[test]
    fn only_official_bundled_cosyvoice_calls_request_gpu_admission() {
        let bundled = json!({"profile": BUNDLED_COSYVOICE_PROFILE_ID});
        assert!(
            bundled_voice_resource_request(OFFICIAL_VOICE_PLUGIN_ID, "voice.warm", &bundled)
                .is_some()
        );
        assert_eq!(
            bundled_voice_resource_request(OFFICIAL_VOICE_PLUGIN_ID, "voice.warm", &bundled)
                .and_then(|request| request.profile_id),
            Some(COSYVOICE_PROFILE_ID.into())
        );
        assert!(
            bundled_voice_resource_request(OFFICIAL_VOICE_PLUGIN_ID, "voice.speak", &bundled)
                .is_some()
        );
        assert!(bundled_voice_resource_request(
            OFFICIAL_VOICE_PLUGIN_ID,
            "voice.speak",
            &json!({"profile": "local-cosyvoice-http"})
        )
        .is_none());
        assert!(bundled_voice_resource_request(
            OFFICIAL_VOICE_PLUGIN_ID,
            "voice.speak",
            &json!({
                "profile": BUNDLED_COSYVOICE_PROFILE_ID,
                "directive": {"synth_profile": "local-cosyvoice-http"}
            })
        )
        .is_none());
        assert!(bundled_voice_resource_request(
            OFFICIAL_VOICE_PLUGIN_ID,
            "voice.speak",
            &json!({
                "profile": "local-cosyvoice-http",
                "directive": {"synth_profile": BUNDLED_COSYVOICE_PROFILE_ID}
            })
        )
        .is_some());
        assert!(
            bundled_voice_resource_request("com.user.voice", "voice.speak", &bundled).is_none()
        );
    }

    #[tokio::test]
    async fn abandoned_cold_start_guard_releases_pending_lease() {
        let coordinator = Arc::new(ResourceCoordinator::new(
            ResourceCoordinatorPolicy::default(),
            Arc::new(UnavailableSnapshot),
        ));
        let request = bundled_voice_resource_request(
            OFFICIAL_VOICE_PLUGIN_ID,
            "voice.warm",
            &json!({"profile": BUNDLED_COSYVOICE_PROFILE_ID}),
        )
        .unwrap();
        let admission = coordinator.admit(request).await;
        let lease_id = admission.lease.unwrap().lease_id;
        {
            let operation_guard = coordinator
                .lock_adapter_operation(COSYVOICE_ADAPTER_ID)
                .await;
            let _guard = DirectoryPluginResourceLease {
                coordinator: Arc::clone(&coordinator),
                lease_id,
                release_on_drop: true,
                _operation_guard: operation_guard,
            };
        }
        assert!(coordinator.diagnostics_snapshot().leases.is_empty());
    }
}

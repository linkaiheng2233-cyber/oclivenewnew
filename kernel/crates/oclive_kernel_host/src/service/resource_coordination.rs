//! Resource Coordinator diagnostics and official runtime-adapter hooks.

use oclive_kernel_types::{
    AppError, ResourceAdapterTransitionRequest, ResourceAdapterTransitionResponse,
    ResourceAdmissionDecision, ResourceAdmissionMode, ResourceAdmissionRequest,
    ResourceAdmissionResult, ResourceControlMode, ResourceCoordinationDiagnostics,
    ResourcePreemptionRecord, ResourcePriority,
};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::domain::resource_coordinator::{configured_gpu_device_index, ResourceCoordinator};
use crate::infrastructure::performance_llm::PerformanceLlmClient;
use crate::infrastructure::resource_adapters::{
    cosyvoice_reservation_mib, COSYVOICE_ADAPTER_ID, COSYVOICE_PROFILE_ID,
    PERFORMANCE_ACTIVITY_ADAPTER_ID,
};
use crate::state::AppState;

const OFFICIAL_VOICE_PLUGIN_ID: &str = "com.oclive.voice.asr";
const BUNDLED_COSYVOICE_PROFILE_ID: &str = "bundled-cosyvoice2-zh";
const COSYVOICE_WORKLOAD_ID: &str = "bundled-runtime";
const EXTERNAL_PERFORMANCE_PREEMPTED_REASON: &str = "external_performance_preempted";

pub enum DirectoryPluginResourceAdmission {
    NotApplicable,
    Admitted(DirectoryPluginResourceLease, u64),
    Denied(Box<ResourceAdmissionResult>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectoryPluginResourceFinalization {
    NotApplicable,
    Released,
    Retained,
    Active,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectoryPluginResourceConfigFinalization {
    NotApplicable,
    Released {
        external_performance_preempted: bool,
    },
    Retained,
}

impl DirectoryPluginResourceAdmission {
    #[must_use]
    pub fn external_performance_preemption_active(&self) -> bool {
        matches!(
            self,
            Self::Admitted(lease, _) if lease.external_performance_preempted
        )
    }

    /// Mark an admission as owned by an authoritative kernel-process
    /// preemption. The directory plugin must unload after this RPC; recovery is
    /// completed by the thin desktop host only after release confirmation.
    pub fn mark_external_performance_preemption(&mut self, params: &mut Value) -> bool {
        let Self::Admitted(lease, reservation_mib) = self else {
            return false;
        };
        lease.external_performance_preempted = true;
        lease.release_after_call = true;
        lease
            .coordinator
            .record_adapter_reason(COSYVOICE_ADAPTER_ID, EXTERNAL_PERFORMANCE_PREEMPTED_REASON);
        inject_resource_admission(params, &lease.lease_id, *reservation_mib, true);
        true
    }
}

/// Host-to-directory-plugin control transition prepared from one config change.
///
/// The host keeps the lease until the plugin reports a matching successful
/// operation; configuration persistence alone is not proof of resource release.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DirectoryPluginResourceConfigAction {
    NotApplicable,
    Unload {
        adapter_id: &'static str,
        runtime_profile_id: &'static str,
    },
}

pub struct DirectoryPluginResourceConfigTransition {
    action: DirectoryPluginResourceConfigAction,
    external_performance_preempted: bool,
    _operation_guard: Option<tokio::sync::OwnedMutexGuard<()>>,
}

impl DirectoryPluginResourceConfigTransition {
    #[must_use]
    pub fn rpc_payload(&self) -> Option<Value> {
        match self.action {
            DirectoryPluginResourceConfigAction::NotApplicable => None,
            DirectoryPluginResourceConfigAction::Unload {
                adapter_id,
                runtime_profile_id,
            } => Some(json!({
                "adapter_id": adapter_id,
                "operation": "unload",
                "runtime_profile_id": runtime_profile_id,
            })),
        }
    }
}

pub struct DirectoryPluginResourceLease {
    coordinator: Arc<ResourceCoordinator>,
    lease_id: String,
    reservation_mib: u64,
    release_on_drop: bool,
    release_after_call: bool,
    external_performance_preempted: bool,
    preempted_performance: Option<Arc<PerformanceLlmClient>>,
    automatic_preemptions: Vec<ResourcePreemptionRecord>,
    _operation_guard: tokio::sync::OwnedMutexGuard<()>,
}

impl DirectoryPluginResourceLease {
    fn commit(&mut self) {
        self.release_on_drop = false;
    }

    fn recover_performance(&mut self, local_provider_selected: bool) {
        let automatic_preemptions = std::mem::take(&mut self.automatic_preemptions);
        if !automatic_preemptions.is_empty() {
            let coordinator = Arc::clone(&self.coordinator);
            tokio::spawn(async move {
                if let Err(error) = coordinator
                    .restore_preempted_adapters(COSYVOICE_ADAPTER_ID, &automatic_preemptions)
                    .await
                {
                    coordinator.record_adapter_reason(
                        COSYVOICE_ADAPTER_ID,
                        "resource_preemption_restore_failed",
                    );
                    tracing::error!(
                        target: "oclive_resource",
                        %error,
                        "automatic resource recovery after bundled voice failed"
                    );
                }
            });
            return;
        }
        let Some(performance) = self.preempted_performance.take() else {
            return;
        };
        if local_provider_selected {
            performance.schedule_warmup();
        }
    }
}

impl Drop for DirectoryPluginResourceLease {
    fn drop(&mut self) {
        if self.release_after_call {
            self.coordinator
                .activate(&self.lease_id, Some(self.reservation_mib));
            self.coordinator
                .record_adapter_reason(COSYVOICE_ADAPTER_ID, "resource_transition_abandoned");
            self.release_on_drop = false;
            return;
        }
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
        ram_reservation_mib: 2_048,
        cpu_thread_reservation: 2,
        priority: if method == "voice.speak" {
            ResourcePriority::ForegroundMedia
        } else {
            ResourcePriority::BackgroundWarmup
        },
        control_mode: ResourceControlMode::Managed,
        admission_mode: ResourceAdmissionMode::Enforced,
    })
}

fn is_local_llm_provider(state: &AppState) -> bool {
    !state
        .user_llm_provider
        .read()
        .trim()
        .eq_ignore_ascii_case("cloud")
}

fn denied_for_gpu_headroom(admission: &ResourceAdmissionResult) -> bool {
    admission.decision == ResourceAdmissionDecision::Denied
        && admission
            .reason_codes
            .iter()
            .any(|reason| reason == "insufficient_gpu_headroom")
}

#[must_use]
pub fn directory_plugin_resource_rpc_needs_kernel_preemption(
    plugin_id: &str,
    method: &str,
    params: &Value,
    admission: &DirectoryPluginResourceAdmission,
) -> bool {
    method == "voice.speak"
        && bundled_voice_resource_request(plugin_id, method, params).is_some()
        && matches!(
            admission,
            DirectoryPluginResourceAdmission::Denied(result)
                if denied_for_gpu_headroom(result)
        )
}

fn should_preempt_performance_for_voice(
    method: &str,
    local_provider_selected: bool,
    admission: &ResourceAdmissionResult,
    performance_request_active: bool,
    performance_suspended: bool,
    releasable_residency: bool,
) -> bool {
    method == "voice.speak"
        && local_provider_selected
        && denied_for_gpu_headroom(admission)
        && !performance_request_active
        && !performance_suspended
        && releasable_residency
}

fn inject_resource_admission(
    params: &mut Value,
    lease_id: &str,
    reservation_mib: u64,
    release_after_call: bool,
) {
    if !params.is_object() {
        *params = json!({});
    }
    if let Some(object) = params.as_object_mut() {
        object.insert(
            "_oclive_resource_admission".into(),
            json!({
                "schema_version": 1,
                "granted": true,
                "lease_id": lease_id,
                "reservation_mib": reservation_mib,
                "release_after_call": release_after_call,
                "preempted_adapter_id": release_after_call
                    .then_some(crate::infrastructure::resource_adapters::LLAMA_RUNTIME_ADAPTER_ID),
            }),
        );
    }
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
    let mut admission = state.resource_coordinator.admit(request.clone()).await;
    let mut preempted_performance = None;
    let performance = state.performance_llm.as_ref();
    if let Some(performance) = performance.filter(|performance| {
        should_preempt_performance_for_voice(
            method,
            is_local_llm_provider(state),
            &admission,
            state
                .resource_coordinator
                .has_active_adapter(PERFORMANCE_ACTIVITY_ADAPTER_ID),
            performance.resource_suspension_active(),
            performance.has_releasable_gpu_residency(),
        )
    }) {
        match performance
            .suspend_managed_runtime_for_resource_pressure(
                "bundled CosyVoice foreground speech requested GPU ownership",
            )
            .await
        {
            Ok(()) => {
                admission = state.resource_coordinator.admit(request).await;
                if admission.decision == ResourceAdmissionDecision::Denied {
                    if is_local_llm_provider(state) {
                        performance.schedule_warmup();
                    }
                } else {
                    preempted_performance = Some(Arc::clone(performance));
                }
            }
            Err(error) => {
                tracing::warn!(
                    target: "oclive_resource",
                    error_code = "VOICE_LLM_PREEMPTION_FAILED",
                    %error,
                    "Performance LLM could not yield GPU ownership to bundled voice"
                );
            }
        }
    }
    if admission.decision == ResourceAdmissionDecision::Denied {
        return DirectoryPluginResourceAdmission::Denied(Box::new(admission));
    }
    let Some(lease_id) = admission.lease.as_ref().map(|lease| lease.lease_id.clone()) else {
        return DirectoryPluginResourceAdmission::Denied(Box::new(admission));
    };
    let external_performance_preempted = admission.lease.as_ref().is_some_and(|lease| {
        lease
            .reason_codes
            .iter()
            .any(|reason| reason == EXTERNAL_PERFORMANCE_PREEMPTED_REASON)
    });
    let automatic_preemptions = admission.preempted_adapters.clone();
    if preempted_performance.is_none() && automatic_preemptions.is_empty() {
        preempted_performance = performance
            .filter(|performance| {
                method == "voice.speak" && performance.resource_suspension_active()
            })
            .map(Arc::clone);
    }
    let release_after_call = preempted_performance.is_some()
        || external_performance_preempted
        || !automatic_preemptions.is_empty();
    inject_resource_admission(params, &lease_id, reservation_mib, release_after_call);
    DirectoryPluginResourceAdmission::Admitted(
        DirectoryPluginResourceLease {
            coordinator: Arc::clone(&state.resource_coordinator),
            lease_id,
            reservation_mib,
            release_on_drop: admission.decision != ResourceAdmissionDecision::Reused,
            release_after_call,
            external_performance_preempted,
            preempted_performance,
            automatic_preemptions,
            _operation_guard: operation_guard,
        },
        reservation_mib,
    )
}

pub fn finalize_directory_plugin_resource_rpc(
    state: &AppState,
    admission: DirectoryPluginResourceAdmission,
    response: Option<&Value>,
) -> DirectoryPluginResourceFinalization {
    let DirectoryPluginResourceAdmission::Admitted(mut lease, reservation_mib) = admission else {
        return DirectoryPluginResourceFinalization::NotApplicable;
    };
    if lease.release_after_call {
        if resource_release_confirmed(COSYVOICE_ADAPTER_ID, response) {
            lease.coordinator.release(&lease.lease_id);
            lease.release_on_drop = false;
            lease.release_after_call = false;
            lease.recover_performance(is_local_llm_provider(state));
            return DirectoryPluginResourceFinalization::Released;
        } else {
            state
                .resource_coordinator
                .activate(&lease.lease_id, Some(reservation_mib));
            lease.commit();
            lease.release_after_call = false;
            lease.preempted_performance = None;
            state
                .resource_coordinator
                .record_adapter_reason(COSYVOICE_ADAPTER_ID, "resource_release_unconfirmed");
            tracing::warn!(
                target: "oclive_resource",
                error_code = "RESOURCE_RELEASE_UNCONFIRMED",
                adapter_id = COSYVOICE_ADAPTER_ID,
                "bundled voice lease retained; Performance LLM remains suspended"
            );
            return DirectoryPluginResourceFinalization::Retained;
        }
    }
    let succeeded = response.and_then(Value::as_object).is_some_and(|result| {
        result.get("ok").and_then(Value::as_bool) == Some(true)
            && result.get("skipped").and_then(Value::as_bool) != Some(true)
    });
    if !succeeded {
        return DirectoryPluginResourceFinalization::Released;
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
    DirectoryPluginResourceFinalization::Active
}

/// Apply an adapter lifecycle transition in the authoritative kernel process.
///
/// This is the cross-process control-plane counterpart to desktop-owned
/// directory-plugin RPC. Every caller/target/operation combination must have an
/// explicit coordinator grant; registering either adapter does not grant control.
///
/// # Errors
///
/// Returns [`AppError::InvalidParameter`] when identifiers, grants, lifecycle
/// support, or profile selection fail. Stale revisions and controller failures
/// preserve their stable unavailable errors.
pub async fn transition_resource_adapter_impl(
    state: &AppState,
    request: &ResourceAdapterTransitionRequest,
) -> Result<ResourceAdapterTransitionResponse, AppError> {
    state.resource_coordinator.transition_adapter(request).await
}

#[must_use]
pub async fn prepare_directory_plugin_resource_config_transition(
    state: &AppState,
    plugin_id: &str,
    config: &Value,
) -> DirectoryPluginResourceConfigTransition {
    let mut action = directory_plugin_resource_config_action(plugin_id, config);
    let operation_guard = if plugin_id == OFFICIAL_VOICE_PLUGIN_ID {
        Some(
            state
                .resource_coordinator
                .lock_adapter_operation(COSYVOICE_ADAPTER_ID)
                .await,
        )
    } else {
        None
    };
    if matches!(action, DirectoryPluginResourceConfigAction::Unload { .. })
        && !state
            .resource_coordinator
            .has_adapter_lease(COSYVOICE_ADAPTER_ID)
    {
        action = DirectoryPluginResourceConfigAction::NotApplicable;
    }
    let external_performance_preempted =
        !matches!(action, DirectoryPluginResourceConfigAction::NotApplicable)
            && state
                .resource_coordinator
                .adapter_has_reason(COSYVOICE_ADAPTER_ID, EXTERNAL_PERFORMANCE_PREEMPTED_REASON);
    DirectoryPluginResourceConfigTransition {
        action,
        external_performance_preempted,
        _operation_guard: operation_guard,
    }
}

fn directory_plugin_resource_config_action(
    plugin_id: &str,
    config: &Value,
) -> DirectoryPluginResourceConfigAction {
    if plugin_id != OFFICIAL_VOICE_PLUGIN_ID {
        return DirectoryPluginResourceConfigAction::NotApplicable;
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
    if enabled && bundled && bundled_profile {
        DirectoryPluginResourceConfigAction::NotApplicable
    } else {
        DirectoryPluginResourceConfigAction::Unload {
            adapter_id: COSYVOICE_ADAPTER_ID,
            runtime_profile_id: BUNDLED_COSYVOICE_PROFILE_ID,
        }
    }
}

pub fn finalize_directory_plugin_resource_config_transition(
    state: &AppState,
    plugin_id: &str,
    transition: DirectoryPluginResourceConfigTransition,
    response: Option<&Value>,
) -> DirectoryPluginResourceConfigFinalization {
    let DirectoryPluginResourceConfigAction::Unload { adapter_id, .. } = transition.action else {
        return DirectoryPluginResourceConfigFinalization::NotApplicable;
    };
    let confirmed = resource_release_confirmed(adapter_id, response);
    if confirmed {
        let released = state.resource_coordinator.release_adapter(adapter_id);
        if let Some(performance) = state
            .performance_llm
            .as_ref()
            .filter(|performance| performance.resource_suspension_active())
        {
            if is_local_llm_provider(state) {
                performance.schedule_warmup();
            }
        }
        tracing::info!(
            target: "oclive_resource",
            plugin_id,
            adapter_id,
            released,
            "directory plugin confirmed resource release"
        );
        DirectoryPluginResourceConfigFinalization::Released {
            external_performance_preempted: transition.external_performance_preempted,
        }
    } else {
        state
            .resource_coordinator
            .record_adapter_reason(adapter_id, "resource_release_unconfirmed");
        tracing::warn!(
            target: "oclive_resource",
            error_code = "RESOURCE_RELEASE_UNCONFIRMED",
            plugin_id,
            adapter_id,
            "directory plugin resource lease retained because runtime release was not confirmed"
        );
        DirectoryPluginResourceConfigFinalization::Retained
    }
}

fn resource_release_confirmed(adapter_id: &str, response: Option<&Value>) -> bool {
    response
        .and_then(|value| value.get("resource_transition"))
        .and_then(Value::as_object)
        .is_some_and(|result| {
            result.get("adapter_id").and_then(Value::as_str) == Some(adapter_id)
                && result.get("operation").and_then(Value::as_str) == Some("unload")
                && result.get("ok").and_then(Value::as_bool) == Some(true)
                && result.get("released").and_then(Value::as_bool) == Some(true)
        })
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

    #[test]
    fn config_transition_requests_unload_only_when_leaving_bundled_runtime() {
        assert_eq!(
            directory_plugin_resource_config_action(
                OFFICIAL_VOICE_PLUGIN_ID,
                &json!({
                    "tts_expansion_enabled": true,
                    "synth_provider": "bundled",
                    "tts_profile": BUNDLED_COSYVOICE_PROFILE_ID,
                }),
            ),
            DirectoryPluginResourceConfigAction::NotApplicable
        );
        assert_eq!(
            directory_plugin_resource_config_action(
                OFFICIAL_VOICE_PLUGIN_ID,
                &json!({
                    "tts_expansion_enabled": false,
                    "synth_provider": "bundled",
                    "tts_profile": BUNDLED_COSYVOICE_PROFILE_ID,
                }),
            ),
            DirectoryPluginResourceConfigAction::Unload {
                adapter_id: COSYVOICE_ADAPTER_ID,
                runtime_profile_id: BUNDLED_COSYVOICE_PROFILE_ID,
            }
        );
        assert_eq!(
            directory_plugin_resource_config_action(
                "com.user.voice",
                &json!({"tts_expansion_enabled": false}),
            ),
            DirectoryPluginResourceConfigAction::NotApplicable
        );
    }

    #[test]
    fn config_transition_payload_is_explicit_and_namespaced() {
        let payload = DirectoryPluginResourceConfigTransition {
            action: DirectoryPluginResourceConfigAction::Unload {
                adapter_id: COSYVOICE_ADAPTER_ID,
                runtime_profile_id: BUNDLED_COSYVOICE_PROFILE_ID,
            },
            external_performance_preempted: false,
            _operation_guard: None,
        }
        .rpc_payload()
        .unwrap();
        assert_eq!(
            payload,
            json!({
                "adapter_id": COSYVOICE_ADAPTER_ID,
                "operation": "unload",
                "runtime_profile_id": BUNDLED_COSYVOICE_PROFILE_ID,
            })
        );
    }

    #[test]
    fn config_transition_requires_matching_successful_release_confirmation() {
        let confirmed = json!({
            "ok": true,
            "resource_transition": {
                "adapter_id": COSYVOICE_ADAPTER_ID,
                "operation": "unload",
                "ok": true,
                "released": true,
            }
        });
        assert!(resource_release_confirmed(
            COSYVOICE_ADAPTER_ID,
            Some(&confirmed)
        ));
        for rejected in [
            json!({}),
            json!({
                "resource_transition": {
                    "adapter_id": "builtin.voice.other",
                    "operation": "unload",
                    "ok": true,
                    "released": true,
                }
            }),
            json!({
                "resource_transition": {
                    "adapter_id": COSYVOICE_ADAPTER_ID,
                    "operation": "unload",
                    "ok": false,
                    "released": true,
                }
            }),
            json!({
                "resource_transition": {
                    "adapter_id": COSYVOICE_ADAPTER_ID,
                    "operation": "unload",
                    "ok": true,
                    "released": false,
                }
            }),
        ] {
            assert!(!resource_release_confirmed(
                COSYVOICE_ADAPTER_ID,
                Some(&rejected)
            ));
        }
        assert!(!resource_release_confirmed(COSYVOICE_ADAPTER_ID, None));
    }

    #[test]
    fn voice_preemption_requires_idle_local_performance_and_headroom_denial() {
        let denied = ResourceAdmissionResult {
            decision: ResourceAdmissionDecision::Denied,
            lease: None,
            snapshot: ResourceSnapshot::unavailable("test", "insufficient_gpu_headroom"),
            pressure: oclive_kernel_types::ResourcePressureLevel::Critical,
            queue_wait_ms: 0,
            preempted_adapters: Vec::new(),
            reason_codes: vec!["insufficient_gpu_headroom".into()],
        };
        assert!(should_preempt_performance_for_voice(
            "voice.speak",
            true,
            &denied,
            false,
            false,
            true,
        ));
        for rejected in [
            ("voice.warm", true, false, false, true),
            ("voice.speak", false, false, false, true),
            ("voice.speak", true, true, false, true),
            ("voice.speak", true, false, true, true),
            ("voice.speak", true, false, false, false),
        ] {
            assert!(!should_preempt_performance_for_voice(
                rejected.0, rejected.1, &denied, rejected.2, rejected.3, rejected.4,
            ));
        }
        let unavailable = ResourceAdmissionResult {
            reason_codes: vec!["gpu_snapshot_unavailable".into()],
            ..denied
        };
        assert!(!should_preempt_performance_for_voice(
            "voice.speak",
            true,
            &unavailable,
            false,
            false,
            true,
        ));
    }

    #[test]
    fn kernel_preemption_is_requested_only_for_denied_bundled_foreground_speech() {
        let denied = DirectoryPluginResourceAdmission::Denied(Box::new(ResourceAdmissionResult {
            decision: ResourceAdmissionDecision::Denied,
            lease: None,
            snapshot: ResourceSnapshot::unavailable("test", "insufficient_gpu_headroom"),
            pressure: oclive_kernel_types::ResourcePressureLevel::Critical,
            queue_wait_ms: 0,
            preempted_adapters: Vec::new(),
            reason_codes: vec!["insufficient_gpu_headroom".into()],
        }));
        assert!(directory_plugin_resource_rpc_needs_kernel_preemption(
            OFFICIAL_VOICE_PLUGIN_ID,
            "voice.speak",
            &json!({"profile": BUNDLED_COSYVOICE_PROFILE_ID}),
            &denied,
        ));
        assert!(!directory_plugin_resource_rpc_needs_kernel_preemption(
            OFFICIAL_VOICE_PLUGIN_ID,
            "voice.warm",
            &json!({"profile": BUNDLED_COSYVOICE_PROFILE_ID}),
            &denied,
        ));
        assert!(!directory_plugin_resource_rpc_needs_kernel_preemption(
            "com.user.voice",
            "voice.speak",
            &json!({"profile": BUNDLED_COSYVOICE_PROFILE_ID}),
            &denied,
        ));
    }

    #[test]
    fn preempted_voice_admission_requires_release_confirmation() {
        let mut params = Value::Null;
        inject_resource_admission(&mut params, "voice-lease", 768, true);
        assert_eq!(
            params.pointer("/_oclive_resource_admission/release_after_call"),
            Some(&Value::Bool(true))
        );
        assert_eq!(
            params.pointer("/_oclive_resource_admission/preempted_adapter_id"),
            Some(&Value::String(
                crate::infrastructure::resource_adapters::LLAMA_RUNTIME_ADAPTER_ID.into()
            ))
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
        let reservation_mib = request.reservation_mib;
        let admission = coordinator.admit(request).await;
        let lease_id = admission.lease.unwrap().lease_id;
        {
            let operation_guard = coordinator
                .lock_adapter_operation(COSYVOICE_ADAPTER_ID)
                .await;
            let _guard = DirectoryPluginResourceLease {
                coordinator: Arc::clone(&coordinator),
                lease_id,
                reservation_mib,
                release_on_drop: true,
                release_after_call: false,
                external_performance_preempted: false,
                preempted_performance: None,
                automatic_preemptions: Vec::new(),
                _operation_guard: operation_guard,
            };
        }
        assert!(coordinator.diagnostics_snapshot().leases.is_empty());
    }

    #[tokio::test]
    async fn abandoned_preempted_voice_call_retains_a_recoverable_lease() {
        let coordinator = Arc::new(ResourceCoordinator::new(
            ResourceCoordinatorPolicy::default(),
            Arc::new(UnavailableSnapshot),
        ));
        let request = bundled_voice_resource_request(
            OFFICIAL_VOICE_PLUGIN_ID,
            "voice.speak",
            &json!({"profile": BUNDLED_COSYVOICE_PROFILE_ID}),
        )
        .unwrap();
        let admission = coordinator.admit(request.clone()).await;
        let lease_id = admission.lease.unwrap().lease_id;
        {
            let operation_guard = coordinator
                .lock_adapter_operation(COSYVOICE_ADAPTER_ID)
                .await;
            let _guard = DirectoryPluginResourceLease {
                coordinator: Arc::clone(&coordinator),
                lease_id,
                reservation_mib: request.reservation_mib,
                release_on_drop: true,
                release_after_call: true,
                external_performance_preempted: true,
                preempted_performance: None,
                automatic_preemptions: Vec::new(),
                _operation_guard: operation_guard,
            };
        }
        let diagnostics = coordinator.diagnostics_snapshot();
        assert_eq!(diagnostics.leases.len(), 1);
        assert_eq!(
            diagnostics.leases[0].state,
            oclive_kernel_types::ResourceLeaseState::Active
        );
        assert!(diagnostics.leases[0]
            .reason_codes
            .iter()
            .any(|reason| reason == "resource_transition_abandoned"));
    }

    #[tokio::test]
    async fn external_preemption_marker_survives_an_unconfirmed_voice_call() {
        let coordinator = Arc::new(ResourceCoordinator::new(
            ResourceCoordinatorPolicy::default(),
            Arc::new(UnavailableSnapshot),
        ));
        let request = bundled_voice_resource_request(
            OFFICIAL_VOICE_PLUGIN_ID,
            "voice.speak",
            &json!({"profile": BUNDLED_COSYVOICE_PROFILE_ID}),
        )
        .unwrap();
        let first = coordinator.admit(request.clone()).await;
        let lease_id = first.lease.expect("voice lease").lease_id;
        let operation_guard = coordinator
            .lock_adapter_operation(COSYVOICE_ADAPTER_ID)
            .await;
        let mut admission = DirectoryPluginResourceAdmission::Admitted(
            DirectoryPluginResourceLease {
                coordinator: Arc::clone(&coordinator),
                lease_id,
                reservation_mib: request.reservation_mib,
                release_on_drop: true,
                release_after_call: false,
                external_performance_preempted: false,
                preempted_performance: None,
                automatic_preemptions: Vec::new(),
                _operation_guard: operation_guard,
            },
            request.reservation_mib,
        );
        let mut params = json!({});
        assert!(admission.mark_external_performance_preemption(&mut params));
        assert!(admission.external_performance_preemption_active());
        drop(admission);

        let reused = coordinator.admit(request).await;
        assert_eq!(reused.decision, ResourceAdmissionDecision::Reused);
        assert!(reused
            .lease
            .expect("reused voice lease")
            .reason_codes
            .iter()
            .any(|reason| reason == EXTERNAL_PERFORMANCE_PREEMPTED_REASON));
    }
}

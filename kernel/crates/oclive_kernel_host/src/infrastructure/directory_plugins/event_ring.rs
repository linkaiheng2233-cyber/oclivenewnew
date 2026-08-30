//! Host-reviewed directory-plugin adapter for the kernel Event Ring.

use std::collections::BTreeSet;
use std::sync::{Arc, Weak};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use oclive_kernel_contracts::{EventModule, EventModuleRegistrar};
use oclive_kernel_types::{
    AppError, EventDraft, EventEnvelope, EventModuleDeclaration, EventModuleFailureMode,
    EventModuleOutput, EventModuleRegistryPolicy, Result,
};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{DirectoryPluginRuntime, OclivePluginManifest};
use crate::domain::event_ring::{EventRing, LEGACY_EVENT_IMPACT_KIND};
use crate::infrastructure::remote_plugin::{
    invoke_directory_plugin_rpc_blocking, RemoteRpcChannel,
};

const DIRECTORY_EVENT_RPC_METHOD: &str = "event_ring.handle";
const DIRECTORY_EVENT_MODULE_PRIORITY: i32 = 1_000;
const DIRECTORY_EVENT_DEFAULT_WEIGHT_BPS: u16 = 5_000;
const DIRECTORY_EVENT_MAX_WEIGHT_BPS: u16 = 8_000;
const DIRECTORY_EVENT_DEFAULT_TIMEOUT_MS: u64 = 2_000;
const DIRECTORY_EVENT_MAX_TIMEOUT_MS: u64 = 5_000;
const DIRECTORY_EVENT_RPC_SCHEMA_VERSION: u16 = 1;
const MAX_CANONICAL_EVENT_KIND_BYTES: usize = 160;

const SAFE_KERNEL_SUBSCRIPTIONS: [&str; 3] = [
    LEGACY_EVENT_IMPACT_KIND,
    oclive_kernel_types::MEMORY_RECALL_CANDIDATE_EVENT_KIND,
    oclive_kernel_types::MEMORY_RECOLLECTION_ACTIVATED_EVENT_KIND,
];

#[derive(Debug, Clone, PartialEq, Eq)]
struct AdmittedDirectoryEventConfig {
    plugin_id: String,
    declaration: EventModuleDeclaration,
    influence_weight_bps: u16,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DirectoryEventRingRequest {
    schema_version: u16,
    event: DirectoryEventView,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DirectoryEventView {
    event_id: String,
    kind: String,
    source: String,
    source_weight_bps: u16,
    correlation_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    causation_id: Option<String>,
    sequence: u64,
    depth: u16,
    occurred_at: DateTime<Utc>,
    payload: Value,
    metadata_keys: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DirectoryEventRingResponse {
    #[serde(default)]
    emitted: Vec<EventDraft>,
}

struct DirectoryEventModule {
    runtime: Weak<DirectoryPluginRuntime>,
    config: AdmittedDirectoryEventConfig,
}

#[async_trait]
impl EventModule for DirectoryEventModule {
    fn declaration(&self) -> EventModuleDeclaration {
        self.config.declaration.clone()
    }

    async fn handle(&self, event: &EventEnvelope) -> Result<EventModuleOutput> {
        let runtime = self.runtime.upgrade().ok_or_else(|| {
            AppError::RemoteServiceUnavailable("directory event runtime unavailable".into())
        })?;
        if runtime
            .effective_slots()
            .is_plugin_disabled(&self.config.plugin_id)
        {
            return Ok(EventModuleOutput::default());
        }
        let request = directory_event_request(event);
        let params = serde_json::to_value(request).map_err(|_| {
            AppError::InvalidParameter("directory event request serialization failed".into())
        })?;
        let plugin_id = self.config.plugin_id.clone();
        let timeout_ms = runtime
            .rpc_timeout_override_ms(&plugin_id, DIRECTORY_EVENT_RPC_METHOD)
            .unwrap_or(DIRECTORY_EVENT_DEFAULT_TIMEOUT_MS)
            .clamp(500, DIRECTORY_EVENT_MAX_TIMEOUT_MS);
        let rpc_runtime = Arc::clone(&runtime);
        let response = tokio::task::spawn_blocking(move || {
            let url = rpc_runtime.ensure_rpc_url(&plugin_id).map_err(|_| {
                AppError::RemoteServiceUnavailable("directory event plugin unavailable".into())
            })?;
            invoke_directory_plugin_rpc_blocking(
                &url,
                DIRECTORY_EVENT_RPC_METHOD,
                params,
                RemoteRpcChannel::Plugin,
                Some(timeout_ms),
            )
        })
        .await
        .map_err(|_| AppError::RemoteServiceUnavailable("directory event task failed".into()))??;
        decode_directory_event_response(response)
    }
}

struct DirectoryEventRingBridge {
    runtime: Weak<DirectoryPluginRuntime>,
    ring: Weak<EventRing>,
    registered_module_ids: Mutex<BTreeSet<String>>,
    sync_lock: Mutex<()>,
}

impl DirectoryEventRingBridge {
    fn sync(&self) {
        let _sync = self.sync_lock.lock();
        let (Some(runtime), Some(ring)) = (self.runtime.upgrade(), self.ring.upgrade()) else {
            return;
        };

        let previous = std::mem::take(&mut *self.registered_module_ids.lock());
        for module_id in previous {
            ring.unregister_event_module(&module_id);
        }

        let mut roots = runtime
            .plugin_roots
            .read()
            .iter()
            .map(|(plugin_id, entry)| (plugin_id.clone(), entry.root.clone()))
            .collect::<Vec<_>>();
        roots.sort_by(|left, right| left.0.cmp(&right.0));

        let mut registered = BTreeSet::new();
        for (plugin_id, root) in roots {
            let manifest = match runtime.load_manifest_cached(&plugin_id, &root) {
                Ok(manifest) => manifest,
                Err(_) => continue,
            };
            if manifest.event_ring.is_none() {
                continue;
            }
            let config = match admit_directory_event_config(&plugin_id, &manifest) {
                Ok(config) => config,
                Err(reason) => {
                    tracing::warn!(
                        target: "oclive_event_ring",
                        error_code = "DIRECTORY_EVENT_MANIFEST_REJECTED",
                        plugin_id,
                        reason,
                        "directory Event Ring declaration rejected"
                    );
                    continue;
                }
            };
            let module_id = config.declaration.module_id.clone();
            let module = Arc::new(DirectoryEventModule {
                runtime: Arc::downgrade(&runtime),
                config: config.clone(),
            });
            let policy = EventModuleRegistryPolicy {
                influence_weight_bps: config.influence_weight_bps,
                failure_mode: EventModuleFailureMode::Isolate,
            };
            match ring.register_event_module_with_policy(module, policy) {
                Ok(_) => {
                    registered.insert(module_id);
                }
                Err(reason) => tracing::warn!(
                    target: "oclive_event_ring",
                    error_code = "DIRECTORY_EVENT_REGISTRATION_FAILED",
                    plugin_id,
                    reason,
                    "directory Event Ring module registration failed"
                ),
            }
        }
        *self.registered_module_ids.lock() = registered;
    }
}

/// Wires currently scanned directory plugins and keeps the registry synchronized after rescans.
pub(crate) fn wire_directory_event_ring(
    runtime: &Arc<DirectoryPluginRuntime>,
    ring: &Arc<EventRing>,
) {
    let bridge = Arc::new(DirectoryEventRingBridge {
        runtime: Arc::downgrade(runtime),
        ring: Arc::downgrade(ring),
        registered_module_ids: Mutex::new(BTreeSet::new()),
        sync_lock: Mutex::new(()),
    });
    let listener_bridge = Arc::clone(&bridge);
    runtime.on_plugin_roots_changed(Arc::new(move || {
        listener_bridge.sync();
    }));
    bridge.sync();
}

fn admit_directory_event_config(
    plugin_id: &str,
    manifest: &OclivePluginManifest,
) -> std::result::Result<AdmittedDirectoryEventConfig, String> {
    if manifest.id != plugin_id {
        return Err("manifest id does not match scanned plugin id".into());
    }
    if plugin_id != plugin_id.to_ascii_lowercase() {
        return Err("Event Ring plugin id must be lowercase".into());
    }
    validate_exact_event_kind(plugin_id)?;
    let section = manifest
        .event_ring
        .as_ref()
        .ok_or_else(|| "eventRing section missing".to_string())?;
    if manifest.process.is_none() {
        return Err("eventRing requires a process section".into());
    }
    if !manifest
        .rpc_methods
        .iter()
        .any(|method| method.trim() == DIRECTORY_EVENT_RPC_METHOD)
    {
        return Err("eventRing requires rpcMethods entry event_ring.handle".into());
    }
    if !oclive_validation::manifest_declares_process_spawn(&manifest.permissions, true) {
        return Err("eventRing process requires process:spawn permission".into());
    }

    let mut subscriptions = BTreeSet::new();
    for kind in &section.subscriptions {
        validate_exact_event_kind(kind)?;
        if !is_host_allowed_subscription(kind) {
            return Err(format!(
                "subscription is outside host-approved ranges: {kind}"
            ));
        }
        subscriptions.insert(kind.clone());
    }
    if subscriptions.is_empty() {
        return Err("eventRing requires at least one approved subscription".into());
    }

    let own_event_prefix = format!("plugin.{plugin_id}");
    let own_event_wildcard = format!("{own_event_prefix}.*");
    let mut emissions = BTreeSet::new();
    for pattern in &section.emissions {
        if pattern == &own_event_wildcard {
            emissions.insert(pattern.clone());
            continue;
        }
        validate_exact_event_kind(pattern)?;
        if !pattern.starts_with(&format!("{own_event_prefix}.")) {
            return Err(format!(
                "emission is outside plugin-owned namespace: {pattern}"
            ));
        }
        emissions.insert(pattern.clone());
    }

    let influence_weight_bps = section
        .suggested_influence_weight_bps
        .unwrap_or(DIRECTORY_EVENT_DEFAULT_WEIGHT_BPS)
        .min(DIRECTORY_EVENT_MAX_WEIGHT_BPS);
    Ok(AdmittedDirectoryEventConfig {
        plugin_id: plugin_id.to_string(),
        declaration: EventModuleDeclaration {
            module_id: format!("directory.{plugin_id}"),
            subscriptions: subscriptions.into_iter().collect(),
            emissions: emissions.into_iter().collect(),
            priority: DIRECTORY_EVENT_MODULE_PRIORITY,
        },
        influence_weight_bps,
    })
}

fn is_host_allowed_subscription(kind: &str) -> bool {
    SAFE_KERNEL_SUBSCRIPTIONS.contains(&kind)
        || (kind.starts_with("plugin.") && kind.split('.').count() >= 3)
}

fn validate_exact_event_kind(kind: &str) -> std::result::Result<(), String> {
    if kind.is_empty()
        || kind != kind.trim()
        || kind.len() > MAX_CANONICAL_EVENT_KIND_BYTES
        || kind.contains('*')
        || kind.split('.').any(|segment| {
            segment.is_empty()
                || !segment.bytes().all(|byte| {
                    byte.is_ascii_lowercase()
                        || byte.is_ascii_digit()
                        || matches!(byte, b'_' | b'-')
                })
        })
    {
        return Err(format!(
            "event kind is not a canonical exact identifier: {kind}"
        ));
    }
    Ok(())
}

fn directory_event_request(event: &EventEnvelope) -> DirectoryEventRingRequest {
    DirectoryEventRingRequest {
        schema_version: DIRECTORY_EVENT_RPC_SCHEMA_VERSION,
        event: DirectoryEventView {
            event_id: event.event_id.clone(),
            kind: event.kind.clone(),
            source: event.source.clone(),
            source_weight_bps: event.source_weight_bps,
            correlation_id: event.correlation_id.clone(),
            causation_id: event.causation_id.clone(),
            sequence: event.sequence,
            depth: event.depth,
            occurred_at: event.occurred_at,
            payload: event.payload.clone(),
            metadata_keys: event.metadata.keys().cloned().collect(),
        },
    }
}

fn decode_directory_event_response(response: Value) -> Result<EventModuleOutput> {
    let response: DirectoryEventRingResponse = serde_json::from_value(response).map_err(|_| {
        AppError::InvalidParameter("directory event response contract invalid".into())
    })?;
    Ok(EventModuleOutput {
        emitted: response.emitted,
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;

    use chrono::TimeZone;
    use oclive_kernel_contracts::{EventEmitter, EventModuleRegistrar};
    use oclive_kernel_types::{
        EventModuleFailureMode, EVENT_RING_SCHEMA_VERSION, MEMORY_RECOLLECTION_ACTIVATED_EVENT_KIND,
    };
    use serde_json::json;

    use super::*;
    use crate::domain::host_profile::HostProfile;
    use crate::infrastructure::high_risk_grants::HighRiskGrantStore;

    struct TestSource;

    #[async_trait]
    impl EventModule for TestSource {
        fn declaration(&self) -> EventModuleDeclaration {
            EventModuleDeclaration {
                module_id: "test.directory_event_source".into(),
                emissions: vec![MEMORY_RECOLLECTION_ACTIVATED_EVENT_KIND.into()],
                ..Default::default()
            }
        }

        async fn handle(&self, _event: &EventEnvelope) -> Result<EventModuleOutput> {
            Ok(EventModuleOutput::default())
        }
    }

    fn write_event_plugin_manifest(root: &std::path::Path, include_event_ring: bool) {
        fs::create_dir_all(root).expect("plugin dir");
        let mut manifest = json!({
            "schema_version": 1,
            "id": "com.example.events",
            "version": "1.0.0",
            "permissions": ["process:spawn"],
            "rpcMethods": [DIRECTORY_EVENT_RPC_METHOD],
            "process": {"command": "not-started-by-registry"}
        });
        if include_event_ring {
            manifest["eventRing"] = json!({
                "subscriptions": [MEMORY_RECOLLECTION_ACTIVATED_EVENT_KIND],
                "emissions": ["plugin.com.example.events.*"],
                "suggestedInfluenceWeightBps": 9_500
            });
        }
        fs::write(root.join("manifest.json"), manifest.to_string()).expect("manifest");
    }

    #[test]
    fn admission_caps_weight_and_rejects_broad_or_foreign_ranges() {
        let temp = tempfile::tempdir().expect("tempdir");
        write_event_plugin_manifest(temp.path(), true);
        let manifest = OclivePluginManifest::load_from_dir(temp.path()).expect("manifest");
        let admitted =
            admit_directory_event_config("com.example.events", &manifest).expect("admitted");
        assert_eq!(
            admitted.influence_weight_bps,
            DIRECTORY_EVENT_MAX_WEIGHT_BPS
        );
        assert_eq!(
            admitted.declaration.priority,
            DIRECTORY_EVENT_MODULE_PRIORITY
        );
        assert_eq!(
            admitted.declaration.emissions,
            vec!["plugin.com.example.events.*"]
        );

        let mut broad = manifest.clone();
        broad.event_ring.as_mut().expect("event ring").subscriptions =
            vec!["kernel.memory.*".into()];
        assert!(admit_directory_event_config("com.example.events", &broad).is_err());

        let mut foreign = manifest;
        foreign.event_ring.as_mut().expect("event ring").emissions =
            vec!["plugin.com.other.events.changed".into()];
        assert!(admit_directory_event_config("com.example.events", &foreign).is_err());
    }

    #[test]
    fn rpc_view_excludes_stream_key_and_metadata_values() {
        let mut metadata = BTreeMap::new();
        metadata.insert("trace.secret".into(), Value::String("private-value".into()));
        let event = EventEnvelope {
            schema_version: EVENT_RING_SCHEMA_VERSION,
            event_id: "event-1".into(),
            kind: MEMORY_RECOLLECTION_ACTIVATED_EVENT_KIND.into(),
            source: "module.builtin.event_decision".into(),
            source_weight_bps: 10_000,
            stream_key: "private-stream".into(),
            correlation_id: "turn-1".into(),
            causation_id: Some("event-0".into()),
            sequence: 2,
            depth: 1,
            occurred_at: Utc.timestamp_opt(0, 0).single().expect("timestamp"),
            payload: json!({"memory_id": "memory-1"}),
            metadata,
        };

        let encoded = serde_json::to_string(&directory_event_request(&event)).expect("request");
        assert!(!encoded.contains("private-stream"));
        assert!(!encoded.contains("private-value"));
        assert!(encoded.contains("trace.secret"));
        assert!(encoded.contains("memory-1"));
    }

    #[test]
    fn rpc_response_cannot_replace_parent_payload_or_metadata() {
        let invalid = json!({
            "payload": {"replace": true},
            "emitted": []
        });
        assert!(decode_directory_event_response(invalid).is_err());
    }

    #[test]
    fn rpc_response_decodes_only_child_event_proposals() {
        let output = decode_directory_event_response(json!({
            "emitted": [{
                "kind": "plugin.com.example.events.recollection_seen",
                "payload": {"memory_id": "memory-1"},
                "metadata": {}
            }]
        }))
        .expect("valid child proposal");
        assert!(output.payload.is_none());
        assert!(output.metadata.is_empty());
        assert_eq!(output.emitted.len(), 1);
        assert_eq!(
            output.emitted[0].kind,
            "plugin.com.example.events.recollection_seen"
        );
    }

    #[test]
    fn initial_scan_and_rescan_add_then_remove_directory_event_module() {
        let temp = tempfile::tempdir().expect("tempdir");
        let roles = temp.path().join("distros/chat-pro/roles");
        let plugin = temp
            .path()
            .join("distros/chat-pro/plugins/com.example.events");
        let app_data = temp.path().join("app-data");
        fs::create_dir_all(&roles).expect("roles");
        fs::create_dir_all(&app_data).expect("app data");
        write_event_plugin_manifest(&plugin, true);
        let grants = HighRiskGrantStore::load(app_data.clone(), false);
        let runtime = DirectoryPluginRuntime::bootstrap_with_host_profile(
            &roles,
            &app_data,
            grants,
            HostProfile::default(),
            false,
        );
        let ring = Arc::new(EventRing::new());

        wire_directory_event_ring(&runtime, &ring);
        assert!(ring.event_module_registry().is_empty());
        runtime.ensure_plugin_roots_scanned();

        let entry = ring
            .event_module_registry()
            .into_iter()
            .find(|entry| entry.declaration.module_id == "directory.com.example.events")
            .expect("directory event module");
        assert_eq!(
            entry.policy.influence_weight_bps,
            DIRECTORY_EVENT_MAX_WEIGHT_BPS
        );
        assert_eq!(entry.policy.failure_mode, EventModuleFailureMode::Isolate);

        write_event_plugin_manifest(&plugin, false);
        runtime.rescan_plugin_roots(&roles);
        assert!(ring
            .event_module_registry()
            .into_iter()
            .all(|entry| entry.declaration.module_id != "directory.com.example.events"));
    }

    #[tokio::test]
    async fn unavailable_directory_process_is_quarantined_without_failing_dispatch() -> Result<()> {
        let temp = tempfile::tempdir().expect("tempdir");
        let roles = temp.path().join("distros/chat-pro/roles");
        let plugin = temp
            .path()
            .join("distros/chat-pro/plugins/com.example.events");
        let app_data = temp.path().join("app-data");
        fs::create_dir_all(&roles).expect("roles");
        fs::create_dir_all(&app_data).expect("app data");
        write_event_plugin_manifest(&plugin, true);
        let grants = HighRiskGrantStore::load(app_data.clone(), true);
        let runtime = DirectoryPluginRuntime::bootstrap_with_host_profile(
            &roles,
            &app_data,
            grants,
            HostProfile::default(),
            true,
        );
        let ring = Arc::new(EventRing::new());
        let source: Arc<dyn EventEmitter> = ring
            .register_event_module(Arc::new(TestSource))
            .map_err(AppError::InvalidParameter)?;
        wire_directory_event_ring(&runtime, &ring);

        for correlation_id in ["directory-failure-1", "directory-failure-2"] {
            let dispatched = source
                .emit(
                    "test-stream",
                    Some(correlation_id),
                    EventDraft {
                        kind: MEMORY_RECOLLECTION_ACTIVATED_EVENT_KIND.into(),
                        payload: json!({"memory_id": "memory-1"}),
                        metadata: BTreeMap::new(),
                    },
                )
                .await?;
            assert_eq!(dispatched.primary.payload, json!({"memory_id": "memory-1"}));
            assert!(dispatched.emitted.is_empty());
        }

        let runtime_diagnostic = ring
            .diagnostics_snapshot(0)
            .module_runtime
            .into_iter()
            .find(|entry| entry.module_id == "directory.com.example.events")
            .expect("directory runtime diagnostic");
        assert!(runtime_diagnostic.quarantined);
        assert_eq!(runtime_diagnostic.failure_count, 1);
        Ok(())
    }
}

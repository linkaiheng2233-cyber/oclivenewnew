use oclive_kernel_types::{ResourceCoordinatorPolicy, RESOURCE_COORDINATION_SCHEMA_VERSION};
use serde_json::{json, Value};

pub fn captured_resources() -> Value {
    let mut policy = serde_json::to_value(ResourceCoordinatorPolicy::default()).unwrap();
    policy["scheduling"] = json!({
        "strategy": "custom",
        "commands": [{"kind": "residency", "adapter_id": "builtin.test", "mode": "resident"}]
    });
    json!({
        "schema_version": RESOURCE_COORDINATION_SCHEMA_VERSION,
        "state_revision": 42,
        "state": "ready", "pressure": "normal", "policy": policy,
        "snapshot": {
            "captured_at_ms": 123, "source": "fixture", "available": true,
            "gpu_devices": [{"device_index": 0, "name": "fixture GPU", "total_mib": 8192,
                "free_mib": 6144, "used_mib": 2048}],
            "system_memory": {"total_mib": 16384, "available_mib": 12288, "used_mib": 4096},
            "cpu": {"logical_cores": 8, "physical_cores": 4}
        },
        "adapters": [{
            "registration_source": "builtin", "registration_source_id": "host",
            "runtime_state": "inactive",
            "descriptor": {
                "adapter_id": "builtin.test", "kind": "runtime", "domain": "llm",
                "control_mode": "managed", "provider_id": "fixture",
                "profiles": [{"profile_id": "gpu", "quality_rank": 100,
                    "execution_target": "gpu", "estimated_reservation_mib": 2048,
                    "requires_restart": true, "coordinator_selectable": true}],
                "lifecycle_operations": ["observe", "start", "unload"],
                "residency_modes": ["resident", "on_demand", "unloaded"]
            }
        }],
        "leases": [],
        // A capture is not an authoritative evaluation of the current profile.
        "scheduling": {"state": "ready", "intent": {"strategy": "compatibility_first"}},
        "candidate_plan": {"plan_id": "stale", "compiled_from_revision": 0,
            "state": "ready", "executable": true}
    })
}

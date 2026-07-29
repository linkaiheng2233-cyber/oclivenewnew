//! Builtin Resource Adapter descriptors.
//!
//! These descriptors report only lifecycle and profile behavior that the
//! current host/runtime path really implements. Automatic profile selection is
//! deliberately left disabled until the coordinator transition engine exists.

use oclive_kernel_types::{
    ResourceAdapterDescriptor, ResourceAdapterDomain, ResourceAdapterKind,
    ResourceAdapterOperation, ResourceControlMode, ResourceExecutionTarget,
    ResourceOperatingProfile, ResourceResidencyMode,
};

pub const LLAMA_RUNTIME_ADAPTER_ID: &str = "builtin.llm.llama_server";
pub const LLAMA_RUNTIME_PROFILE_ID: &str = "configured";
pub const OLLAMA_ADAPTER_ID: &str = "builtin.llm.ollama";
pub const OLLAMA_PROFILE_ID: &str = "external";
pub const PERFORMANCE_ACTIVITY_ADAPTER_ID: &str = "builtin.llm.performance_request";
pub const COSYVOICE_ADAPTER_ID: &str = "builtin.voice.cosyvoice2";
pub const COSYVOICE_PROFILE_ID: &str = "bundled_auto_precision";

pub const ENV_COSYVOICE_GPU_RESERVATION_MIB: &str = "OCLIVE_COSYVOICE_GPU_RESERVATION_MIB";
pub const DEFAULT_COSYVOICE_GPU_RESERVATION_MIB: u64 = 768;

#[must_use]
pub fn cosyvoice_reservation_mib() -> u64 {
    std::env::var(ENV_COSYVOICE_GPU_RESERVATION_MIB)
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .unwrap_or(DEFAULT_COSYVOICE_GPU_RESERVATION_MIB)
        .min(65_536)
}

#[must_use]
pub fn llama_server_descriptor() -> ResourceAdapterDescriptor {
    let gpu_layers = std::env::var("OCLIVE_LLAMA_GPU_LAYERS")
        .ok()
        .and_then(|value| value.trim().parse::<i64>().ok())
        .unwrap_or(0);
    let estimated_reservation_mib = std::env::var("OCLIVE_LLAMA_GPU_RESERVATION_MIB")
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .map(|value| value.min(65_536));
    ResourceAdapterDescriptor {
        adapter_id: LLAMA_RUNTIME_ADAPTER_ID.into(),
        kind: ResourceAdapterKind::Runtime,
        domain: ResourceAdapterDomain::Llm,
        provider_id: Some("builtin.llm.performance".into()),
        control_mode: ResourceControlMode::Managed,
        profiles: vec![ResourceOperatingProfile {
            profile_id: LLAMA_RUNTIME_PROFILE_ID.into(),
            quality_rank: 100,
            execution_target: if gpu_layers > 0 {
                ResourceExecutionTarget::Gpu
            } else {
                ResourceExecutionTarget::Cpu
            },
            estimated_reservation_mib,
            requires_restart: true,
            coordinator_selectable: false,
        }],
        lifecycle_operations: vec![
            ResourceAdapterOperation::Start,
            ResourceAdapterOperation::Resume,
            ResourceAdapterOperation::Suspend,
            ResourceAdapterOperation::Unload,
            ResourceAdapterOperation::Release,
        ],
        residency_modes: vec![
            ResourceResidencyMode::Resident,
            ResourceResidencyMode::OnDemand,
            ResourceResidencyMode::Suspended,
            ResourceResidencyMode::Unloaded,
        ],
    }
}

#[must_use]
pub fn ollama_descriptor() -> ResourceAdapterDescriptor {
    ResourceAdapterDescriptor {
        adapter_id: OLLAMA_ADAPTER_ID.into(),
        kind: ResourceAdapterKind::Runtime,
        domain: ResourceAdapterDomain::Llm,
        provider_id: Some("builtin.llm.ollama".into()),
        control_mode: ResourceControlMode::ObserveOnly,
        profiles: vec![ResourceOperatingProfile {
            profile_id: OLLAMA_PROFILE_ID.into(),
            quality_rank: 50,
            execution_target: ResourceExecutionTarget::External,
            estimated_reservation_mib: None,
            requires_restart: false,
            coordinator_selectable: false,
        }],
        lifecycle_operations: vec![ResourceAdapterOperation::Observe],
        residency_modes: vec![ResourceResidencyMode::External],
    }
}

#[must_use]
pub fn performance_activity_descriptor() -> ResourceAdapterDescriptor {
    ResourceAdapterDescriptor {
        adapter_id: PERFORMANCE_ACTIVITY_ADAPTER_ID.into(),
        kind: ResourceAdapterKind::ActivityObserver,
        domain: ResourceAdapterDomain::Llm,
        provider_id: Some("builtin.llm.performance".into()),
        control_mode: ResourceControlMode::ObserveOnly,
        profiles: Vec::new(),
        lifecycle_operations: vec![ResourceAdapterOperation::Observe],
        residency_modes: Vec::new(),
    }
}

#[must_use]
pub fn cosyvoice_descriptor() -> ResourceAdapterDescriptor {
    ResourceAdapterDescriptor {
        adapter_id: COSYVOICE_ADAPTER_ID.into(),
        kind: ResourceAdapterKind::Runtime,
        domain: ResourceAdapterDomain::Voice,
        provider_id: Some("com.oclive.voice.asr".into()),
        control_mode: ResourceControlMode::Managed,
        profiles: vec![ResourceOperatingProfile {
            profile_id: COSYVOICE_PROFILE_ID.into(),
            quality_rank: 100,
            execution_target: ResourceExecutionTarget::Gpu,
            estimated_reservation_mib: Some(cosyvoice_reservation_mib()),
            requires_restart: true,
            coordinator_selectable: false,
        }],
        lifecycle_operations: vec![
            ResourceAdapterOperation::Start,
            ResourceAdapterOperation::Unload,
            ResourceAdapterOperation::Release,
        ],
        residency_modes: vec![
            ResourceResidencyMode::Resident,
            ResourceResidencyMode::OnDemand,
            ResourceResidencyMode::Unloaded,
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observe_only_ollama_does_not_claim_managed_operations() {
        let descriptor = ollama_descriptor();
        assert_eq!(descriptor.control_mode, ResourceControlMode::ObserveOnly);
        assert_eq!(
            descriptor.lifecycle_operations,
            vec![ResourceAdapterOperation::Observe]
        );
        assert!(!descriptor.profiles[0].coordinator_selectable);
    }

    #[test]
    fn managed_descriptors_do_not_claim_automatic_profile_switching() {
        for descriptor in [llama_server_descriptor(), cosyvoice_descriptor()] {
            assert!(descriptor
                .profiles
                .iter()
                .all(|profile| !profile.coordinator_selectable));
        }
    }
}

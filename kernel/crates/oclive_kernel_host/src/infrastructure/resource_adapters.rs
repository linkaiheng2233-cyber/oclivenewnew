//! Builtin Resource Adapter descriptors.
//!
//! These descriptors report only lifecycle and profile behavior that the
//! current host/runtime path really implements.

use oclive_kernel_types::{
    ResourceAdapterDescriptor, ResourceAdapterDomain, ResourceAdapterKind,
    ResourceAdapterOperation, ResourceControlMode, ResourceExecutionTarget,
    ResourceOperatingProfile, ResourceResidencyMode,
};

pub const LLAMA_RUNTIME_ADAPTER_ID: &str = "builtin.llm.llama_server";
pub const LLAMA_RUNTIME_PROFILE_FULL: &str = "gpu_full";
pub const LLAMA_RUNTIME_PROFILE_BALANCED: &str = "gpu_balanced";
pub const LLAMA_RUNTIME_PROFILE_COMPATIBILITY: &str = "cpu_compatibility";
pub const ENV_LLAMA_PERFORMANCE_PROFILE: &str = "OCLIVE_LLAMA_PERFORMANCE_PROFILE";
pub const OLLAMA_ADAPTER_ID: &str = "builtin.llm.ollama";
pub const OLLAMA_PROFILE_ID: &str = "external";
pub const PERFORMANCE_ACTIVITY_ADAPTER_ID: &str = "builtin.llm.performance_request";
pub const COSYVOICE_ADAPTER_ID: &str = "builtin.voice.cosyvoice2";
pub const COSYVOICE_PROFILE_ID: &str = "bundled_auto_precision";

pub const ENV_COSYVOICE_GPU_RESERVATION_MIB: &str = "OCLIVE_COSYVOICE_GPU_RESERVATION_MIB";
const DEFAULT_LLAMA_BALANCED_GPU_LAYERS: i32 = 22;
/// Cold-load reservation measured against the bundled mixed-FP16 staged loader.
/// The sidecar reports roughly 1.3 GiB peak on the supported Windows profile;
/// keep headroom above that peak instead of treating the steady-state footprint
/// as the admission estimate.
pub const DEFAULT_COSYVOICE_GPU_RESERVATION_MIB: u64 = 1_536;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LlamaRuntimeTier {
    pub profile_id: &'static str,
    pub gpu_layers: i32,
}

impl LlamaRuntimeTier {
    #[must_use]
    pub fn execution_target(self) -> ResourceExecutionTarget {
        if self.gpu_layers > 0 {
            ResourceExecutionTarget::Hybrid
        } else {
            ResourceExecutionTarget::Cpu
        }
    }
}

#[must_use]
pub fn configured_llama_tiers() -> [LlamaRuntimeTier; 3] {
    let full_layers = std::env::var("OCLIVE_LLAMA_GPU_LAYERS")
        .ok()
        .and_then(|value| value.trim().parse::<i32>().ok())
        .unwrap_or(99)
        // Keep `gpu_full`, `gpu_balanced`, and `cpu_compatibility` materially
        // distinct even when a development environment supplies a tiny value.
        .clamp(2, 999);
    let balanced_layers = if full_layers > DEFAULT_LLAMA_BALANCED_GPU_LAYERS {
        DEFAULT_LLAMA_BALANCED_GPU_LAYERS
    } else {
        (full_layers / 2).max(1)
    };
    [
        LlamaRuntimeTier {
            profile_id: LLAMA_RUNTIME_PROFILE_FULL,
            gpu_layers: full_layers,
        },
        LlamaRuntimeTier {
            profile_id: LLAMA_RUNTIME_PROFILE_BALANCED,
            gpu_layers: balanced_layers,
        },
        LlamaRuntimeTier {
            profile_id: LLAMA_RUNTIME_PROFILE_COMPATIBILITY,
            gpu_layers: 0,
        },
    ]
}

#[must_use]
pub fn llama_tier(profile_id: &str) -> Option<LlamaRuntimeTier> {
    configured_llama_tiers()
        .into_iter()
        .find(|tier| tier.profile_id == profile_id)
}

#[must_use]
pub fn configured_llama_tier() -> LlamaRuntimeTier {
    configured_llama_tier_with_default(LLAMA_RUNTIME_PROFILE_BALANCED)
}

#[must_use]
pub fn configured_llama_tier_with_default(default_profile_id: &str) -> LlamaRuntimeTier {
    std::env::var(ENV_LLAMA_PERFORMANCE_PROFILE)
        .ok()
        .and_then(|profile_id| llama_tier(profile_id.trim()))
        .or_else(|| llama_tier(default_profile_id))
        .unwrap_or_else(|| {
            configured_llama_tiers()
                .into_iter()
                .find(|tier| tier.profile_id == LLAMA_RUNTIME_PROFILE_BALANCED)
                .unwrap_or(LlamaRuntimeTier {
                    profile_id: LLAMA_RUNTIME_PROFILE_COMPATIBILITY,
                    gpu_layers: 0,
                })
        })
}

fn configured_model_mib() -> Option<u64> {
    let path = std::env::var("OCLIVE_LOCAL_LLM_MODEL_PATH").ok()?;
    std::fs::metadata(path.trim())
        .ok()
        .map(|metadata| metadata.len().saturating_add(1024 * 1024 - 1) / (1024 * 1024))
}

#[must_use]
pub fn llama_tier_gpu_reservation_mib(tier: LlamaRuntimeTier) -> u64 {
    if tier.gpu_layers <= 0 {
        return 0;
    }
    if tier.profile_id == LLAMA_RUNTIME_PROFILE_FULL {
        if let Some(value) = std::env::var("OCLIVE_LLAMA_GPU_RESERVATION_MIB")
            .ok()
            .and_then(|value| value.trim().parse::<u64>().ok())
        {
            return value.min(65_536);
        }
    }
    configured_model_mib().map_or(0, |model_mib| {
        (model_mib
            .saturating_mul((tier.gpu_layers as u64).min(32))
            .saturating_add(31)
            / 32)
            .saturating_add(512)
            .min(65_536)
    })
}

#[must_use]
pub fn llama_host_ram_reservation_mib() -> Option<u64> {
    configured_model_mib().map(|model_mib| model_mib.saturating_add(512).min(65_536))
}

#[must_use]
pub fn llama_tiers_from(start: LlamaRuntimeTier) -> Vec<LlamaRuntimeTier> {
    let tiers = configured_llama_tiers();
    tiers
        .into_iter()
        .skip_while(|tier| tier.profile_id != start.profile_id)
        .collect()
}

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
    let host_ram_mib = llama_host_ram_reservation_mib();
    ResourceAdapterDescriptor {
        adapter_id: LLAMA_RUNTIME_ADAPTER_ID.into(),
        kind: ResourceAdapterKind::Runtime,
        domain: ResourceAdapterDomain::Llm,
        provider_id: Some("builtin.llm.performance".into()),
        control_mode: ResourceControlMode::Managed,
        profiles: configured_llama_tiers()
            .into_iter()
            .enumerate()
            .map(|(index, tier)| ResourceOperatingProfile {
                profile_id: tier.profile_id.into(),
                quality_rank: [100, 70, 30][index],
                execution_target: tier.execution_target(),
                estimated_reservation_mib: Some(llama_tier_gpu_reservation_mib(tier)),
                estimated_ram_mib: host_ram_mib,
                estimated_cpu_threads: Some(if tier.gpu_layers > 0 { 2 } else { 4 }),
                requires_restart: true,
                coordinator_selectable: true,
            })
            .collect(),
        lifecycle_operations: vec![
            ResourceAdapterOperation::Start,
            ResourceAdapterOperation::Resume,
            ResourceAdapterOperation::Suspend,
        ],
        residency_modes: vec![
            ResourceResidencyMode::Resident,
            ResourceResidencyMode::OnDemand,
            ResourceResidencyMode::Suspended,
            ResourceResidencyMode::Unloaded,
        ],
        automatic_preemption: Some(ResourceAdapterOperation::Suspend),
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
            estimated_ram_mib: None,
            estimated_cpu_threads: None,
            requires_restart: false,
            coordinator_selectable: false,
        }],
        lifecycle_operations: vec![ResourceAdapterOperation::Observe],
        residency_modes: vec![ResourceResidencyMode::External],
        automatic_preemption: None,
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
        automatic_preemption: None,
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
            estimated_ram_mib: Some(2_048),
            estimated_cpu_threads: Some(2),
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
        automatic_preemption: Some(ResourceAdapterOperation::Unload),
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
    fn only_host_controlled_llama_profile_is_selectable() {
        let llama = llama_server_descriptor();
        assert_eq!(llama.profiles.len(), 3);
        assert!(llama
            .profiles
            .iter()
            .all(|profile| profile.coordinator_selectable));
        assert!(!cosyvoice_descriptor().profiles[0].coordinator_selectable);
    }

    #[test]
    fn llama_profiles_change_real_gpu_layer_counts() {
        let tiers = configured_llama_tiers();
        assert!(tiers[0].gpu_layers > tiers[1].gpu_layers);
        assert!(tiers[1].gpu_layers > tiers[2].gpu_layers);
        assert!(tiers[1].gpu_layers <= DEFAULT_LLAMA_BALANCED_GPU_LAYERS);
        assert_eq!(tiers[2].gpu_layers, 0);
    }
}

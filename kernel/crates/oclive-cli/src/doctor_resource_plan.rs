//! Offline resource preview. Captured facts never confer controller authority.

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct ResourcePlanArgs {
    /// Captured ResourceCoordinationDiagnostics JSON, not a role pack or execution request.
    pub diagnostics_file: PathBuf,

    /// Evaluate this distro profile's resource policy instead of the captured policy.
    #[arg(long)]
    pub distro_profile: Option<PathBuf>,

    /// Select a GPU in the captured snapshot (default: lowest captured device index).
    #[arg(long)]
    pub gpu_device_index: Option<u32>,

    /// Print one canonical ResourceCandidatePlan JSON document to stdout.
    #[arg(long)]
    pub json: bool,
}

#[cfg(feature = "diagnostics-host")]
pub(crate) fn format_resource_reason(code: &str) -> String {
    let explanation = match code {
        "resource_plan_controller_unavailable" => {
            "No live controller is available for a proposed transition"
        }
        "resource_plan_insufficient_gpu_headroom" => {
            "Captured GPU headroom is below the proposed demand plus safety reserve"
        }
        "resource_plan_insufficient_system_memory_headroom" => {
            "Captured RAM headroom is below the proposed demand plus safety reserve"
        }
        "resource_plan_insufficient_cpu_thread_headroom" => {
            "Captured CPU thread headroom is below the proposed demand plus safety reserve"
        }
        "resource_plan_gpu_capacity_unknown" => {
            "A selected GPU or hybrid profile has no GPU reservation estimate"
        }
        "resource_plan_capacity_unknown" => {
            "A proposed start or resume lacks a usable capacity estimate"
        }
        "resource_plan_gpu_capacity_unverified" => {
            "The capture has no verified GPU capacity snapshot"
        }
        "resource_plan_gpu_device_unavailable" => {
            "The selected GPU device is absent from the captured snapshot"
        }
        "resource_plan_system_memory_capacity_unverified" => {
            "The capture has no system memory snapshot"
        }
        "resource_plan_cpu_capacity_unverified" => "The capture has no CPU capacity snapshot",
        "resource_plan_no_selectable_profile" => {
            "A referenced adapter has no coordinator-selectable profile"
        }
        "resource_plan_start_unavailable" => {
            "A required nonresident adapter has no proposed start or resume operation"
        }
        "resource_plan_rollback_unavailable" => "A proposed transition has no rollback operation",
        "resource_custom_schedule_empty" => {
            "The custom strategy requires at least one scheduling command"
        }
        "resource_primary_adapter_required" => {
            "The primary-first strategy requires a primary adapter ID"
        }
        "resource_scheduling_group_conflict" => {
            "At least two adapters are required to both coexist and be exclusive"
        }
        "resource_scheduling_residency_conflict" => {
            "An adapter has conflicting residency preferences"
        }
        "resource_scheduling_control_unavailable" => {
            "Exclusive scheduling requires control that an adapter does not advertise"
        }
        _ => "No built-in explanation is available",
    };
    format!("{code} — {explanation}")
}

#[cfg(feature = "diagnostics-host")]
pub(crate) fn preview(
    args: &ResourcePlanArgs,
) -> Result<(
    oclive_kernel_types::ResourceCoordinationDiagnostics,
    oclive_kernel_types::ResourceCandidatePlan,
)> {
    use anyhow::{bail, Context};
    use oclive_kernel_host::domain::host_profile::load_host_profile_file;
    use oclive_kernel_host::domain::resource_adapter_registry::ResourceAdapterRegistry;
    use oclive_kernel_host::domain::resource_plan::{
        compile_resource_candidate_plan, CompileResourceCandidatePlanInput,
    };
    use oclive_kernel_types::{
        ResourceAdapterRegistration, ResourceCoordinationDiagnostics,
        RESOURCE_COORDINATION_SCHEMA_VERSION,
    };
    use std::collections::BTreeSet;

    let bytes = std::fs::read(&args.diagnostics_file).with_context(|| {
        format!(
            "read resource diagnostics {}",
            args.diagnostics_file.display()
        )
    })?;
    let mut capture: ResourceCoordinationDiagnostics =
        serde_json::from_slice(&bytes).context("parse captured ResourceCoordinationDiagnostics")?;
    if capture.schema_version != RESOURCE_COORDINATION_SCHEMA_VERSION {
        bail!(
            "resource diagnostics schema_version {} is unsupported; expected {}",
            capture.schema_version,
            RESOURCE_COORDINATION_SCHEMA_VERSION
        );
    }
    if let Some(path) = &args.distro_profile {
        capture.policy = load_host_profile_file(path)
            .map_err(|error| anyhow::anyhow!("load distro profile {}: {error}", path.display()))?
            .resource_coordination;
    }
    let registry = ResourceAdapterRegistry::new();
    let mut ids = BTreeSet::new();
    for adapter in &capture.adapters {
        if !ids.insert(adapter.descriptor.adapter_id.as_str()) {
            bail!(
                "duplicate captured resource adapter {}",
                adapter.descriptor.adapter_id
            );
        }
        registry
            .register_owned(ResourceAdapterRegistration {
                source: adapter.registration_source,
                source_id: adapter.registration_source_id.clone(),
                descriptor: adapter.descriptor.clone(),
            })
            .map_err(anyhow::Error::msg)?;
    }
    // Revalidate the effective intent. Captured scheduling/candidate results may be stale.
    capture.scheduling = registry.scheduling_diagnostics(&capture.policy.scheduling);
    let plan = compile_resource_candidate_plan(&CompileResourceCandidatePlanInput {
        state_revision: capture.state_revision,
        policy: &capture.policy,
        snapshot: &capture.snapshot,
        gpu_device_index: args.gpu_device_index,
        adapters: &capture.adapters,
        leases: &capture.leases,
        scheduling: &capture.scheduling,
        // Descriptors and capture files cannot establish a live single-writer controller.
        controller_ids: &BTreeSet::new(),
    });
    Ok((capture, plan))
}

pub fn run(args: ResourcePlanArgs) -> Result<()> {
    #[cfg(feature = "diagnostics-host")]
    {
        let (capture, plan) = preview(&args)?;
        eprintln!("offline resource preview: captured facts only; no live controllers or execution authority");
        if args.json {
            println!("{}", serde_json::to_string_pretty(&plan)?);
        } else {
            println!("oclive doctor resource-plan — offline preview");
            println!(
                "  snapshot: {} captured_at_ms={} revision={}",
                capture.snapshot.source,
                capture.snapshot.captured_at_ms,
                plan.compiled_from_revision
            );
            println!(
                "  strategy: {:?}; GPU reserve MiB: {}",
                capture.policy.scheduling.strategy, capture.policy.gpu_safety_reserve_mib
            );
            println!(
                "  state: {:?}; executable (captured eligibility only): {}",
                plan.state, plan.executable
            );
            for adapter in &capture.adapters {
                println!(
                    "  adapter {}: {:?}",
                    adapter.descriptor.adapter_id, adapter.descriptor.control_mode
                );
                for profile in &adapter.descriptor.profiles {
                    println!(
                        "    profile {}: {:?} gpu_mib={:?} ram_mib={:?} cpu_threads={:?} selectable={}",
                        profile.profile_id, profile.execution_target,
                        profile.estimated_reservation_mib, profile.estimated_ram_mib,
                        profile.estimated_cpu_threads, profile.coordinator_selectable
                    );
                }
            }
            for selection in &plan.selections {
                println!(
                    "  selected {}: {} ({:?})",
                    selection.adapter_id, selection.profile_id, selection.source
                );
            }
            for transition in &plan.transitions {
                println!(
                    "  proposed {}: {:?}; rollback={:?}",
                    transition.adapter_id, transition.operation, transition.rollback_operation
                );
            }
            for reason in &plan.reason_codes {
                let reason = format_resource_reason(reason);
                println!("  reason: {reason}");
            }
        }
        Ok(())
    }
    #[cfg(not(feature = "diagnostics-host"))]
    {
        let _ = args;
        anyhow::bail!(
            "`doctor resource-plan` requires building oclive-cli with feature `diagnostics-host`"
        );
    }
}

#[cfg(all(test, feature = "diagnostics-host"))]
mod resource_reason_tests {
    #[test]
    fn unknown_reason_stays_intact_without_inventing_a_meaning() {
        let code = "resource_future_reason_v17";
        assert_eq!(
            super::format_resource_reason(code),
            format!("{code} — No built-in explanation is available")
        );
    }
}

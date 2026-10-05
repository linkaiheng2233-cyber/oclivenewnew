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
fn preview(
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
    let scheduling = registry.scheduling_diagnostics(&capture.policy.scheduling);
    let plan = compile_resource_candidate_plan(&CompileResourceCandidatePlanInput {
        state_revision: capture.state_revision,
        policy: &capture.policy,
        snapshot: &capture.snapshot,
        gpu_device_index: args.gpu_device_index,
        adapters: &capture.adapters,
        leases: &capture.leases,
        scheduling: &scheduling,
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

//! Pure, read-only compilation of resource scheduling intent into a candidate plan.
//!
//! The compiler never calls an adapter and never mutates a lease. Its output is
//! a diagnostic proposal that must pass revision and controller checks again at
//! execution time.

use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};

use oclive_kernel_types::{
    ResourceAdapterDiagnostic, ResourceAdapterOperation, ResourceAdapterRuntimeState,
    ResourceCandidatePlan, ResourceCandidatePlanState, ResourceCandidateTransition,
    ResourceCoordinatorPolicy, ResourceExecutionTarget, ResourceLeaseDiagnostic,
    ResourceProfileSelection, ResourceProfileSelectionSource, ResourceResidencyPreference,
    ResourceSchedulingCommand, ResourceSchedulingIntentDiagnostics, ResourceSchedulingIntentState,
    ResourceSchedulingStrategy, ResourceSnapshot,
};
use sha2::{Digest, Sha256};

pub struct CompileResourceCandidatePlanInput<'a> {
    pub state_revision: u64,
    pub policy: &'a ResourceCoordinatorPolicy,
    pub snapshot: &'a ResourceSnapshot,
    pub gpu_device_index: Option<u32>,
    pub adapters: &'a [ResourceAdapterDiagnostic],
    pub leases: &'a [ResourceLeaseDiagnostic],
    pub scheduling: &'a ResourceSchedulingIntentDiagnostics,
    pub controller_ids: &'a BTreeSet<String>,
}

#[must_use]
pub fn compile_resource_candidate_plan(
    input: &CompileResourceCandidatePlanInput<'_>,
) -> ResourceCandidatePlan {
    let mut blocked_reasons = BTreeSet::new();
    let mut degraded_reasons = BTreeSet::new();
    match input.scheduling.state {
        ResourceSchedulingIntentState::Ready => {}
        ResourceSchedulingIntentState::Degraded => {
            degraded_reasons.extend(input.scheduling.reason_codes.iter().cloned());
        }
        ResourceSchedulingIntentState::Blocked => {
            blocked_reasons.extend(input.scheduling.reason_codes.iter().cloned());
        }
    }

    let adapters = input
        .adapters
        .iter()
        .map(|adapter| (adapter.descriptor.adapter_id.as_str(), adapter))
        .collect::<BTreeMap<_, _>>();
    let mut referenced = BTreeSet::new();
    if let Some(primary) = input.scheduling.intent.primary_adapter_id.as_deref() {
        referenced.insert(primary.to_string());
    }
    for command in &input.scheduling.intent.commands {
        collect_referenced_adapters(command, &mut referenced);
    }

    let fallback_by_adapter = input
        .scheduling
        .intent
        .commands
        .iter()
        .filter_map(|command| match command {
            ResourceSchedulingCommand::Fallback {
                adapter_id,
                profile_ids,
            } => Some((adapter_id.as_str(), profile_ids.as_slice())),
            _ => None,
        })
        .collect::<BTreeMap<_, _>>();

    let mut selections = Vec::new();
    let mut has_unselected_reference = false;
    for adapter in input.adapters {
        let adapter_id = adapter.descriptor.adapter_id.as_str();
        let should_select = referenced.contains(adapter_id) || adapter.current_profile_id.is_some();
        if !should_select || adapter.descriptor.profiles.is_empty() {
            continue;
        }
        let fallback = fallback_by_adapter.get(adapter_id).copied();
        match select_profile(adapter, fallback, input.scheduling.intent.strategy) {
            Some(selection) => selections.push(selection),
            None if referenced.contains(adapter_id) => {
                degraded_reasons.insert("resource_plan_no_selectable_profile".into());
                has_unselected_reference = true;
            }
            None => {}
        }
    }
    selections.sort_by(|left, right| left.adapter_id.cmp(&right.adapter_id));

    let selections_by_adapter = selections
        .iter()
        .map(|selection| (selection.adapter_id.as_str(), selection))
        .collect::<BTreeMap<_, _>>();
    let mut transitions = Vec::new();
    let mut required_start_adapters = BTreeSet::new();

    for command in &input.scheduling.intent.commands {
        match command {
            ResourceSchedulingCommand::YieldThenRun {
                yielding_adapter_id,
                target_adapter_id,
            } => {
                required_start_adapters.insert(target_adapter_id.clone());
                if let Some(yielding) = adapters.get(yielding_adapter_id.as_str()) {
                    if adapter_is_resident(yielding) {
                        push_yield_transition(
                            &mut transitions,
                            yielding,
                            Some(target_adapter_id.clone()),
                            "resource_plan_yield_before_run",
                        );
                    }
                }
                if let Some(target) = adapters.get(target_adapter_id.as_str()) {
                    if !adapter_is_resident(target) {
                        push_start_transition(
                            &mut transitions,
                            target,
                            selections_by_adapter
                                .get(target_adapter_id.as_str())
                                .copied(),
                            Some(yielding_adapter_id.clone()),
                            "resource_plan_run_after_yield",
                        );
                    }
                }
            }
            ResourceSchedulingCommand::Residency {
                adapter_id,
                mode: ResourceResidencyPreference::Resident,
            } => {
                required_start_adapters.insert(adapter_id.clone());
                if let Some(adapter) = adapters.get(adapter_id.as_str()) {
                    if !adapter_is_resident(adapter) {
                        push_start_transition(
                            &mut transitions,
                            adapter,
                            selections_by_adapter.get(adapter_id.as_str()).copied(),
                            None,
                            "resource_plan_residency_requested",
                        );
                    }
                }
            }
            ResourceSchedulingCommand::Exclusive { adapter_ids } => {
                let active = adapter_ids
                    .iter()
                    .filter_map(|adapter_id| adapters.get(adapter_id.as_str()).copied())
                    .filter(|adapter| adapter_is_resident(adapter))
                    .collect::<Vec<_>>();
                if active.len() > 1 {
                    let survivor = input
                        .scheduling
                        .intent
                        .primary_adapter_id
                        .as_deref()
                        .filter(|primary| {
                            active
                                .iter()
                                .any(|item| item.descriptor.adapter_id == *primary)
                        })
                        .unwrap_or(active[0].descriptor.adapter_id.as_str());
                    for adapter in active
                        .into_iter()
                        .filter(|adapter| adapter.descriptor.adapter_id != survivor)
                    {
                        push_yield_transition(
                            &mut transitions,
                            adapter,
                            Some(survivor.to_string()),
                            "resource_plan_exclusive_group",
                        );
                    }
                }
            }
            ResourceSchedulingCommand::Require { .. }
            | ResourceSchedulingCommand::Residency {
                mode: ResourceResidencyPreference::OnDemand,
                ..
            }
            | ResourceSchedulingCommand::Coexist { .. }
            | ResourceSchedulingCommand::Fallback { .. } => {}
        }
    }

    if input.scheduling.intent.strategy == ResourceSchedulingStrategy::PrimaryFirst {
        if let Some(primary_id) = input.scheduling.intent.primary_adapter_id.as_deref() {
            required_start_adapters.insert(primary_id.to_string());
            if let Some(primary) = adapters.get(primary_id) {
                if !adapter_is_resident(primary) {
                    push_start_transition(
                        &mut transitions,
                        primary,
                        selections_by_adapter.get(primary_id).copied(),
                        None,
                        "resource_plan_primary_first",
                    );
                }
            }
        }
    }

    for selection in &selections {
        let Some(adapter) = adapters.get(selection.adapter_id.as_str()) else {
            continue;
        };
        let profile_changed = adapter.current_profile_id.as_deref() != Some(&selection.profile_id);
        if profile_changed && adapter_is_resident(adapter) {
            push_profile_transition(&mut transitions, adapter, selection);
        }
    }
    deduplicate_transitions(&mut transitions);
    let has_unexecutable_start = required_start_adapters.iter().any(|adapter_id| {
        adapters.get(adapter_id.as_str()).is_some_and(|adapter| {
            !adapter_is_resident(adapter)
                && !transitions.iter().any(|transition| {
                    transition.adapter_id == *adapter_id
                        && matches!(
                            transition.operation,
                            ResourceAdapterOperation::Start | ResourceAdapterOperation::Resume
                        )
                })
        })
    });
    if has_unexecutable_start {
        degraded_reasons.insert("resource_plan_start_unavailable".into());
    }

    for transition in &transitions {
        if !input.controller_ids.contains(&transition.adapter_id) {
            degraded_reasons.insert("resource_plan_controller_unavailable".into());
        }
        if transition.rollback_operation.is_none() {
            degraded_reasons.insert("resource_plan_rollback_unavailable".into());
        }
    }
    validate_capacity(
        input,
        &adapters,
        &transitions,
        &mut degraded_reasons,
        &mut blocked_reasons,
    );

    let (state, reason_codes) = if blocked_reasons.is_empty() {
        if degraded_reasons.is_empty() {
            (ResourceCandidatePlanState::Ready, Vec::new())
        } else {
            (
                ResourceCandidatePlanState::Degraded,
                degraded_reasons.into_iter().collect(),
            )
        }
    } else {
        blocked_reasons.extend(degraded_reasons);
        (
            ResourceCandidatePlanState::Blocked,
            blocked_reasons.into_iter().collect(),
        )
    };
    let executable = state != ResourceCandidatePlanState::Blocked
        && !has_unselected_reference
        && !has_unexecutable_start
        && transitions.iter().all(|transition| {
            input.controller_ids.contains(&transition.adapter_id)
                && transition.rollback_operation.is_some()
        });
    let plan_id = candidate_plan_id(
        input.state_revision,
        &input.scheduling.intent,
        &selections,
        &transitions,
    );
    ResourceCandidatePlan {
        plan_id,
        compiled_from_revision: input.state_revision,
        state,
        executable,
        selections,
        transitions,
        reason_codes,
    }
}

fn collect_referenced_adapters(
    command: &ResourceSchedulingCommand,
    referenced: &mut BTreeSet<String>,
) {
    match command {
        ResourceSchedulingCommand::Require { adapter_id }
        | ResourceSchedulingCommand::Residency { adapter_id, .. }
        | ResourceSchedulingCommand::Fallback { adapter_id, .. } => {
            referenced.insert(adapter_id.clone());
        }
        ResourceSchedulingCommand::Coexist { adapter_ids }
        | ResourceSchedulingCommand::Exclusive { adapter_ids } => {
            referenced.extend(adapter_ids.iter().cloned());
        }
        ResourceSchedulingCommand::YieldThenRun {
            yielding_adapter_id,
            target_adapter_id,
        } => {
            referenced.insert(yielding_adapter_id.clone());
            referenced.insert(target_adapter_id.clone());
        }
    }
}

fn select_profile(
    adapter: &ResourceAdapterDiagnostic,
    fallback: Option<&[String]>,
    strategy: ResourceSchedulingStrategy,
) -> Option<ResourceProfileSelection> {
    if let Some(profile_ids) = fallback {
        if let Some(current) = adapter
            .current_profile_id
            .as_deref()
            .filter(|current| profile_ids.iter().any(|profile_id| profile_id == current))
        {
            let profile = adapter
                .descriptor
                .profiles
                .iter()
                .find(|profile| profile.profile_id == current)?;
            return Some(profile_selection(
                adapter,
                profile,
                ResourceProfileSelectionSource::Current,
            ));
        }
        for profile_id in profile_ids {
            if let Some(profile) =
                adapter.descriptor.profiles.iter().find(|profile| {
                    profile.profile_id == *profile_id && profile.coordinator_selectable
                })
            {
                return Some(profile_selection(
                    adapter,
                    profile,
                    ResourceProfileSelectionSource::Fallback,
                ));
            }
        }
        return None;
    }

    if let Some(current) = adapter.current_profile_id.as_deref() {
        if let Some(profile) = adapter
            .descriptor
            .profiles
            .iter()
            .find(|profile| profile.profile_id == current)
        {
            return Some(profile_selection(
                adapter,
                profile,
                ResourceProfileSelectionSource::Current,
            ));
        }
    }

    let mut candidates = adapter
        .descriptor
        .profiles
        .iter()
        .filter(|profile| profile.coordinator_selectable)
        .collect::<Vec<_>>();
    match strategy {
        ResourceSchedulingStrategy::LatencyFirst => candidates.sort_by_key(|profile| {
            (
                match profile.execution_target {
                    ResourceExecutionTarget::Gpu => 0,
                    ResourceExecutionTarget::Hybrid => 1,
                    ResourceExecutionTarget::Cpu => 2,
                    ResourceExecutionTarget::External => 3,
                },
                profile.requires_restart,
                Reverse(profile.quality_rank),
                profile.profile_id.as_str(),
            )
        }),
        ResourceSchedulingStrategy::CompatibilityFirst => candidates.sort_by_key(|profile| {
            (
                profile.estimated_reservation_mib.unwrap_or(u64::MAX),
                profile.estimated_ram_mib.unwrap_or(u64::MAX),
                profile.estimated_cpu_threads.unwrap_or(u16::MAX),
                Reverse(profile.quality_rank),
                profile.profile_id.as_str(),
            )
        }),
        ResourceSchedulingStrategy::PrimaryFirst | ResourceSchedulingStrategy::Custom => candidates
            .sort_by_key(|profile| (Reverse(profile.quality_rank), profile.profile_id.as_str())),
    }
    candidates.first().map(|profile| {
        profile_selection(adapter, profile, ResourceProfileSelectionSource::Strategy)
    })
}

fn profile_selection(
    adapter: &ResourceAdapterDiagnostic,
    profile: &oclive_kernel_types::ResourceOperatingProfile,
    source: ResourceProfileSelectionSource,
) -> ResourceProfileSelection {
    ResourceProfileSelection {
        adapter_id: adapter.descriptor.adapter_id.clone(),
        profile_id: profile.profile_id.clone(),
        source,
        estimated_reservation_mib: profile.estimated_reservation_mib,
        estimated_ram_mib: profile.estimated_ram_mib,
        estimated_cpu_threads: profile.estimated_cpu_threads,
    }
}

fn adapter_is_resident(adapter: &ResourceAdapterDiagnostic) -> bool {
    matches!(
        adapter.runtime_state,
        ResourceAdapterRuntimeState::Active | ResourceAdapterRuntimeState::Reserved
    )
}

fn push_yield_transition(
    transitions: &mut Vec<ResourceCandidateTransition>,
    adapter: &ResourceAdapterDiagnostic,
    requested_by_adapter_id: Option<String>,
    reason_code: &str,
) {
    let (operation, rollback_operation) = if adapter
        .descriptor
        .lifecycle_operations
        .contains(&ResourceAdapterOperation::Suspend)
        && adapter
            .descriptor
            .lifecycle_operations
            .contains(&ResourceAdapterOperation::Resume)
    {
        (
            ResourceAdapterOperation::Suspend,
            Some(ResourceAdapterOperation::Resume),
        )
    } else if adapter
        .descriptor
        .lifecycle_operations
        .contains(&ResourceAdapterOperation::Unload)
        && adapter
            .descriptor
            .lifecycle_operations
            .contains(&ResourceAdapterOperation::Start)
    {
        (
            ResourceAdapterOperation::Unload,
            Some(ResourceAdapterOperation::Start),
        )
    } else {
        return;
    };
    transitions.push(ResourceCandidateTransition {
        adapter_id: adapter.descriptor.adapter_id.clone(),
        operation,
        profile_id: adapter.current_profile_id.clone(),
        rollback_operation,
        rollback_profile_id: adapter.current_profile_id.clone(),
        requested_by_adapter_id,
        reason_codes: vec![reason_code.into()],
    });
}

fn push_start_transition(
    transitions: &mut Vec<ResourceCandidateTransition>,
    adapter: &ResourceAdapterDiagnostic,
    selection: Option<&ResourceProfileSelection>,
    requested_by_adapter_id: Option<String>,
    reason_code: &str,
) {
    if !adapter.descriptor.profiles.is_empty() && selection.is_none() {
        return;
    }
    let operation = if adapter
        .descriptor
        .lifecycle_operations
        .contains(&ResourceAdapterOperation::Start)
    {
        ResourceAdapterOperation::Start
    } else if adapter
        .descriptor
        .lifecycle_operations
        .contains(&ResourceAdapterOperation::Resume)
    {
        ResourceAdapterOperation::Resume
    } else {
        return;
    };
    let rollback_operation = adapter
        .descriptor
        .lifecycle_operations
        .contains(&ResourceAdapterOperation::Suspend)
        .then_some(ResourceAdapterOperation::Suspend)
        .or_else(|| {
            adapter
                .descriptor
                .lifecycle_operations
                .contains(&ResourceAdapterOperation::Unload)
                .then_some(ResourceAdapterOperation::Unload)
        });
    transitions.push(ResourceCandidateTransition {
        adapter_id: adapter.descriptor.adapter_id.clone(),
        operation,
        profile_id: selection.map(|selection| selection.profile_id.clone()),
        rollback_operation,
        rollback_profile_id: None,
        requested_by_adapter_id,
        reason_codes: vec![reason_code.into()],
    });
}

fn push_profile_transition(
    transitions: &mut Vec<ResourceCandidateTransition>,
    adapter: &ResourceAdapterDiagnostic,
    selection: &ResourceProfileSelection,
) {
    if !adapter
        .descriptor
        .lifecycle_operations
        .contains(&ResourceAdapterOperation::Start)
    {
        return;
    }
    transitions.push(ResourceCandidateTransition {
        adapter_id: adapter.descriptor.adapter_id.clone(),
        operation: ResourceAdapterOperation::Start,
        profile_id: Some(selection.profile_id.clone()),
        rollback_operation: adapter
            .current_profile_id
            .as_ref()
            .map(|_| ResourceAdapterOperation::Start),
        rollback_profile_id: adapter.current_profile_id.clone(),
        requested_by_adapter_id: None,
        reason_codes: vec!["resource_plan_profile_change".into()],
    });
}

fn deduplicate_transitions(transitions: &mut Vec<ResourceCandidateTransition>) {
    let mut seen = BTreeSet::new();
    transitions.retain(|transition| {
        seen.insert((
            transition.adapter_id.clone(),
            transition.operation,
            transition.profile_id.clone(),
        ))
    });
}

fn validate_capacity(
    input: &CompileResourceCandidatePlanInput<'_>,
    adapters: &BTreeMap<&str, &ResourceAdapterDiagnostic>,
    transitions: &[ResourceCandidateTransition],
    degraded_reasons: &mut BTreeSet<String>,
    blocked_reasons: &mut BTreeSet<String>,
) {
    let mut requested_gpu_mib = 0_u64;
    let mut requested_ram_mib = 0_u64;
    let mut requested_cpu_threads = 0_u64;
    let mut released_gpu_mib = 0_u64;
    let mut released_ram_mib = 0_u64;
    let mut released_cpu_threads = 0_u64;
    let mut gpu_capacity_relevant = false;
    let mut ram_capacity_relevant = false;
    let mut cpu_capacity_relevant = false;
    for transition in transitions {
        match transition.operation {
            ResourceAdapterOperation::Start | ResourceAdapterOperation::Resume => {
                let Some(adapter) = adapters.get(transition.adapter_id.as_str()) else {
                    continue;
                };
                let profile = transition.profile_id.as_deref().and_then(|profile_id| {
                    adapter
                        .descriptor
                        .profiles
                        .iter()
                        .find(|profile| profile.profile_id == profile_id)
                });
                if profile.is_some_and(|profile| {
                    matches!(
                        profile.execution_target,
                        ResourceExecutionTarget::Gpu | ResourceExecutionTarget::Hybrid
                    )
                }) {
                    gpu_capacity_relevant = true;
                    match profile.and_then(|profile| profile.estimated_reservation_mib) {
                        Some(estimate) => {
                            requested_gpu_mib = requested_gpu_mib.saturating_add(estimate);
                        }
                        None => {
                            degraded_reasons.insert("resource_plan_gpu_capacity_unknown".into());
                        }
                    }
                }
                if let Some(estimate) = profile.and_then(|profile| profile.estimated_ram_mib) {
                    ram_capacity_relevant = true;
                    requested_ram_mib = requested_ram_mib.saturating_add(estimate);
                }
                if let Some(estimate) = profile.and_then(|profile| profile.estimated_cpu_threads) {
                    cpu_capacity_relevant = true;
                    requested_cpu_threads =
                        requested_cpu_threads.saturating_add(u64::from(estimate));
                }
                if profile.is_none()
                    || profile.is_some_and(|profile| {
                        profile.execution_target != ResourceExecutionTarget::External
                            && profile.estimated_reservation_mib.is_none()
                            && profile.estimated_ram_mib.is_none()
                            && profile.estimated_cpu_threads.is_none()
                    })
                {
                    degraded_reasons.insert("resource_plan_capacity_unknown".into());
                }
            }
            ResourceAdapterOperation::Suspend
            | ResourceAdapterOperation::Unload
            | ResourceAdapterOperation::Release => {
                for lease in input
                    .leases
                    .iter()
                    .filter(|lease| lease.adapter_id == transition.adapter_id)
                {
                    released_gpu_mib = released_gpu_mib
                        .saturating_add(lease.actual_mib.max(lease.reservation_mib));
                    released_ram_mib = released_ram_mib
                        .saturating_add(lease.actual_ram_mib.max(lease.ram_reservation_mib));
                    released_cpu_threads = released_cpu_threads.saturating_add(u64::from(
                        lease.actual_cpu_threads.max(lease.cpu_thread_reservation),
                    ));
                }
            }
            ResourceAdapterOperation::Observe => {}
        }
    }
    if gpu_capacity_relevant {
        if !input.snapshot.available {
            degraded_reasons.insert("resource_plan_gpu_capacity_unverified".into());
        } else {
            let free_mib = input.gpu_device_index.map_or_else(
                || {
                    input
                        .snapshot
                        .gpu_devices
                        .iter()
                        .min_by_key(|device| device.device_index)
                        .map(|device| device.free_mib)
                },
                |requested| {
                    input
                        .snapshot
                        .gpu_devices
                        .iter()
                        .find(|device| device.device_index == requested)
                        .map(|device| device.free_mib)
                },
            );
            match free_mib {
                Some(free_mib)
                    if free_mib.saturating_add(released_gpu_mib)
                        < input
                            .policy
                            .gpu_safety_reserve_mib
                            .saturating_add(requested_gpu_mib) =>
                {
                    blocked_reasons.insert("resource_plan_insufficient_gpu_headroom".into());
                }
                Some(_) => {}
                None => {
                    blocked_reasons.insert("resource_plan_gpu_device_unavailable".into());
                }
            }
        }
    }
    if ram_capacity_relevant {
        match input.snapshot.system_memory.as_ref() {
            Some(memory)
                if memory.available_mib.saturating_add(released_ram_mib)
                    < input
                        .policy
                        .system_memory_safety_reserve_mib
                        .saturating_add(requested_ram_mib) =>
            {
                blocked_reasons.insert("resource_plan_insufficient_system_memory_headroom".into());
            }
            Some(_) => {}
            None => {
                degraded_reasons.insert("resource_plan_system_memory_capacity_unverified".into());
            }
        }
    }
    if cpu_capacity_relevant {
        match input.snapshot.cpu.as_ref() {
            Some(cpu) => {
                let occupied = input
                    .leases
                    .iter()
                    .map(|lease| {
                        u64::from(lease.actual_cpu_threads.max(lease.cpu_thread_reservation))
                    })
                    .sum::<u64>();
                let available_after_yield = u64::from(cpu.logical_cores)
                    .saturating_sub(occupied)
                    .saturating_add(released_cpu_threads);
                if available_after_yield
                    < u64::from(input.policy.cpu_safety_reserve_threads)
                        .saturating_add(requested_cpu_threads)
                {
                    blocked_reasons.insert("resource_plan_insufficient_cpu_thread_headroom".into());
                }
            }
            None => {
                degraded_reasons.insert("resource_plan_cpu_capacity_unverified".into());
            }
        }
    }
}

fn candidate_plan_id(
    revision: u64,
    intent: &oclive_kernel_types::ResourceSchedulingIntent,
    selections: &[ResourceProfileSelection],
    transitions: &[ResourceCandidateTransition],
) -> String {
    let payload = serde_json::to_vec(&(revision, intent, selections, transitions))
        .unwrap_or_else(|_| revision.to_le_bytes().to_vec());
    let digest = Sha256::digest(payload);
    format!("resource-plan-{}", hex_prefix(&digest, 8))
}

fn hex_prefix(bytes: &[u8], length: usize) -> String {
    bytes
        .iter()
        .take(length)
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use oclive_kernel_types::{
        CpuSnapshot, GpuDeviceSnapshot, ResourceAdapterDescriptor, ResourceAdapterDomain,
        ResourceAdapterKind, ResourceControlMode, ResourceLeaseState, ResourceOperatingProfile,
        ResourcePriority, ResourceResidencyMode, ResourceSchedulingIntent, SystemMemorySnapshot,
    };

    fn profile(
        profile_id: &str,
        quality_rank: u16,
        estimate: Option<u64>,
        selectable: bool,
    ) -> ResourceOperatingProfile {
        ResourceOperatingProfile {
            profile_id: profile_id.into(),
            quality_rank,
            execution_target: ResourceExecutionTarget::Gpu,
            estimated_reservation_mib: estimate,
            estimated_ram_mib: None,
            estimated_cpu_threads: None,
            requires_restart: true,
            coordinator_selectable: selectable,
        }
    }

    fn adapter(
        adapter_id: &str,
        profiles: Vec<ResourceOperatingProfile>,
        runtime_state: ResourceAdapterRuntimeState,
        current_profile_id: Option<&str>,
    ) -> ResourceAdapterDiagnostic {
        ResourceAdapterDiagnostic {
            descriptor: ResourceAdapterDescriptor {
                adapter_id: adapter_id.into(),
                kind: ResourceAdapterKind::Runtime,
                domain: ResourceAdapterDomain::Llm,
                provider_id: Some("test".into()),
                control_mode: ResourceControlMode::Managed,
                profiles,
                lifecycle_operations: vec![
                    ResourceAdapterOperation::Start,
                    ResourceAdapterOperation::Resume,
                    ResourceAdapterOperation::Suspend,
                ],
                residency_modes: vec![
                    ResourceResidencyMode::Resident,
                    ResourceResidencyMode::OnDemand,
                    ResourceResidencyMode::Suspended,
                ],
                automatic_preemption: Some(ResourceAdapterOperation::Suspend),
            },
            registration_source: oclive_kernel_types::ResourceAdapterRegistrationSource::Builtin,
            registration_source_id: "host".into(),
            runtime_state,
            current_profile_id: current_profile_id.map(str::to_string),
            lease_ids: Vec::new(),
            reason_codes: Vec::new(),
        }
    }

    fn snapshot(free_mib: u64) -> ResourceSnapshot {
        ResourceSnapshot {
            captured_at_ms: 1,
            source: "test".into(),
            available: true,
            gpu_devices: vec![GpuDeviceSnapshot {
                device_index: 0,
                name: "test".into(),
                total_mib: 8192,
                free_mib,
                used_mib: 8192_u64.saturating_sub(free_mib),
            }],
            system_memory: None,
            cpu: None,
            reason_codes: Vec::new(),
        }
    }

    fn snapshot_with_host_resources(
        free_mib: u64,
        available_ram_mib: u64,
        logical_cores: u16,
    ) -> ResourceSnapshot {
        let mut snapshot = snapshot(free_mib);
        snapshot.system_memory = Some(SystemMemorySnapshot {
            total_mib: 16_384,
            available_mib: available_ram_mib,
            used_mib: 16_384_u64.saturating_sub(available_ram_mib),
        });
        snapshot.cpu = Some(CpuSnapshot {
            logical_cores,
            physical_cores: Some(logical_cores / 2),
        });
        snapshot
    }

    fn lease(adapter_id: &str, actual_mib: u64) -> ResourceLeaseDiagnostic {
        ResourceLeaseDiagnostic {
            lease_id: format!("{adapter_id}-lease"),
            adapter_id: adapter_id.into(),
            workload_id: "test".into(),
            profile_id: Some("full".into()),
            gpu_device_index: Some(0),
            reservation_mib: actual_mib,
            actual_mib,
            ram_reservation_mib: 0,
            actual_ram_mib: 0,
            cpu_thread_reservation: 0,
            actual_cpu_threads: 0,
            priority: ResourcePriority::Resident,
            control_mode: ResourceControlMode::Managed,
            state: ResourceLeaseState::Active,
            granted_at_ms: 1,
            expires_at_ms: None,
            reason_codes: Vec::new(),
        }
    }

    fn ready_scheduling(intent: ResourceSchedulingIntent) -> ResourceSchedulingIntentDiagnostics {
        ResourceSchedulingIntentDiagnostics {
            state: ResourceSchedulingIntentState::Ready,
            intent,
            reason_codes: Vec::new(),
        }
    }

    #[test]
    fn fallback_order_selects_first_controllable_profile() {
        let adapters = vec![adapter(
            "llm",
            vec![
                profile("full", 100, Some(4096), false),
                profile("balanced", 80, Some(3072), true),
                profile("economy", 50, Some(2048), true),
            ],
            ResourceAdapterRuntimeState::Inactive,
            None,
        )];
        let scheduling = ResourceSchedulingIntentDiagnostics {
            state: ResourceSchedulingIntentState::Degraded,
            intent: ResourceSchedulingIntent {
                strategy: ResourceSchedulingStrategy::Custom,
                primary_adapter_id: None,
                commands: vec![ResourceSchedulingCommand::Fallback {
                    adapter_id: "llm".into(),
                    profile_ids: vec!["full".into(), "economy".into(), "balanced".into()],
                }],
            },
            reason_codes: vec!["resource_profile_not_coordinator_selectable".into()],
        };
        let policy = ResourceCoordinatorPolicy {
            scheduling: scheduling.intent.clone(),
            ..ResourceCoordinatorPolicy::default()
        };
        let controllers = BTreeSet::from(["llm".to_string()]);
        let plan = compile_resource_candidate_plan(&CompileResourceCandidatePlanInput {
            state_revision: 7,
            policy: &policy,
            snapshot: &snapshot(7000),
            gpu_device_index: Some(0),
            adapters: &adapters,
            leases: &[],
            scheduling: &scheduling,
            controller_ids: &controllers,
        });
        assert_eq!(plan.state, ResourceCandidatePlanState::Degraded);
        assert_eq!(plan.selections[0].profile_id, "economy");
        assert_eq!(
            plan.selections[0].source,
            ResourceProfileSelectionSource::Fallback
        );
        assert!(plan
            .reason_codes
            .iter()
            .any(|reason| reason == "resource_profile_not_coordinator_selectable"));
    }

    #[test]
    fn yield_then_run_accounts_for_confirmed_residency_release() {
        let adapters = vec![
            adapter(
                "llm",
                vec![profile("full", 100, Some(4096), true)],
                ResourceAdapterRuntimeState::Active,
                Some("full"),
            ),
            adapter(
                "voice",
                vec![profile("full", 100, Some(1536), true)],
                ResourceAdapterRuntimeState::Inactive,
                None,
            ),
        ];
        let scheduling = ready_scheduling(ResourceSchedulingIntent {
            strategy: ResourceSchedulingStrategy::Custom,
            primary_adapter_id: None,
            commands: vec![ResourceSchedulingCommand::YieldThenRun {
                yielding_adapter_id: "llm".into(),
                target_adapter_id: "voice".into(),
            }],
        });
        let policy = ResourceCoordinatorPolicy {
            scheduling: scheduling.intent.clone(),
            ..ResourceCoordinatorPolicy::default()
        };
        let controllers = BTreeSet::from(["llm".to_string(), "voice".to_string()]);
        let leases = vec![lease("llm", 4096)];
        let plan = compile_resource_candidate_plan(&CompileResourceCandidatePlanInput {
            state_revision: 11,
            policy: &policy,
            snapshot: &snapshot(512),
            gpu_device_index: Some(0),
            adapters: &adapters,
            leases: &leases,
            scheduling: &scheduling,
            controller_ids: &controllers,
        });
        assert_eq!(plan.state, ResourceCandidatePlanState::Ready);
        assert!(plan.executable);
        assert_eq!(plan.transitions.len(), 2);
        assert_eq!(
            plan.transitions[0].operation,
            ResourceAdapterOperation::Suspend
        );
        assert_eq!(
            plan.transitions[1].operation,
            ResourceAdapterOperation::Start
        );
    }

    #[test]
    fn missing_controller_degrades_preview_without_claiming_execution() {
        let adapters = vec![adapter(
            "llm",
            vec![profile("full", 100, Some(4096), true)],
            ResourceAdapterRuntimeState::Inactive,
            None,
        )];
        let scheduling = ready_scheduling(ResourceSchedulingIntent {
            strategy: ResourceSchedulingStrategy::PrimaryFirst,
            primary_adapter_id: Some("llm".into()),
            commands: Vec::new(),
        });
        let policy = ResourceCoordinatorPolicy {
            scheduling: scheduling.intent.clone(),
            ..ResourceCoordinatorPolicy::default()
        };
        let plan = compile_resource_candidate_plan(&CompileResourceCandidatePlanInput {
            state_revision: 2,
            policy: &policy,
            snapshot: &snapshot(7000),
            gpu_device_index: Some(0),
            adapters: &adapters,
            leases: &[],
            scheduling: &scheduling,
            controller_ids: &BTreeSet::new(),
        });
        assert_eq!(plan.state, ResourceCandidatePlanState::Degraded);
        assert!(!plan.executable);
        assert!(plan
            .reason_codes
            .iter()
            .any(|reason| reason == "resource_plan_controller_unavailable"));
    }

    #[test]
    fn insufficient_capacity_blocks_candidate_before_execution() {
        let adapters = vec![adapter(
            "voice",
            vec![profile("full", 100, Some(1536), true)],
            ResourceAdapterRuntimeState::Inactive,
            None,
        )];
        let scheduling = ready_scheduling(ResourceSchedulingIntent {
            strategy: ResourceSchedulingStrategy::Custom,
            primary_adapter_id: None,
            commands: vec![ResourceSchedulingCommand::Residency {
                adapter_id: "voice".into(),
                mode: ResourceResidencyPreference::Resident,
            }],
        });
        let policy = ResourceCoordinatorPolicy {
            scheduling: scheduling.intent.clone(),
            ..ResourceCoordinatorPolicy::default()
        };
        let controllers = BTreeSet::from(["voice".to_string()]);
        let plan = compile_resource_candidate_plan(&CompileResourceCandidatePlanInput {
            state_revision: 3,
            policy: &policy,
            snapshot: &snapshot(1000),
            gpu_device_index: Some(0),
            adapters: &adapters,
            leases: &[],
            scheduling: &scheduling,
            controller_ids: &controllers,
        });
        assert_eq!(plan.state, ResourceCandidatePlanState::Blocked);
        assert!(!plan.executable);
        assert!(plan
            .reason_codes
            .iter()
            .any(|reason| reason == "resource_plan_insufficient_gpu_headroom"));
    }

    #[test]
    fn render_hybrid_profile_is_checked_against_ram_and_cpu_capacity() {
        let mut render_profile = profile("gpu_full", 100, Some(512), true);
        render_profile.execution_target = ResourceExecutionTarget::Hybrid;
        render_profile.estimated_ram_mib = Some(2_000);
        render_profile.estimated_cpu_threads = Some(2);
        let mut render = adapter(
            "com.example.live2d.render",
            vec![render_profile],
            ResourceAdapterRuntimeState::Inactive,
            None,
        );
        render.descriptor.domain = ResourceAdapterDomain::Render;
        let adapters = vec![render];
        let controllers = ["com.example.live2d.render".into()].into_iter().collect();
        let policy = ResourceCoordinatorPolicy {
            scheduling: ResourceSchedulingIntent {
                strategy: ResourceSchedulingStrategy::LatencyFirst,
                primary_adapter_id: None,
                commands: vec![ResourceSchedulingCommand::Residency {
                    adapter_id: "com.example.live2d.render".into(),
                    mode: ResourceResidencyPreference::Resident,
                }],
            },
            ..ResourceCoordinatorPolicy::default()
        };
        let snapshot = snapshot_with_host_resources(6_000, 2_500, 8);
        let scheduling = ready_scheduling(policy.scheduling.clone());
        let plan = compile_resource_candidate_plan(&CompileResourceCandidatePlanInput {
            state_revision: 1,
            policy: &policy,
            snapshot: &snapshot,
            gpu_device_index: Some(0),
            adapters: &adapters,
            leases: &[],
            scheduling: &scheduling,
            controller_ids: &controllers,
        });
        assert_eq!(plan.state, ResourceCandidatePlanState::Blocked);
        assert!(plan
            .reason_codes
            .iter()
            .any(|reason| reason == "resource_plan_insufficient_system_memory_headroom"));
        assert_eq!(plan.selections[0].estimated_ram_mib, Some(2_000));
        assert_eq!(plan.selections[0].estimated_cpu_threads, Some(2));
    }

    #[test]
    fn current_nonselectable_profile_is_preserved_for_compatibility() {
        let adapters = vec![adapter(
            "external",
            vec![profile("current", 100, None, false)],
            ResourceAdapterRuntimeState::Active,
            Some("current"),
        )];
        let scheduling = ready_scheduling(ResourceSchedulingIntent::default());
        let policy = ResourceCoordinatorPolicy::default();
        let plan = compile_resource_candidate_plan(&CompileResourceCandidatePlanInput {
            state_revision: 4,
            policy: &policy,
            snapshot: &snapshot(5000),
            gpu_device_index: Some(0),
            adapters: &adapters,
            leases: &[],
            scheduling: &scheduling,
            controller_ids: &BTreeSet::new(),
        });
        assert_eq!(plan.state, ResourceCandidatePlanState::Ready);
        assert!(plan.executable);
        assert_eq!(plan.transitions.len(), 0);
        assert_eq!(
            plan.selections[0].source,
            ResourceProfileSelectionSource::Current
        );
    }

    #[test]
    fn inactive_adapter_without_selectable_profile_never_claims_executable_start() {
        let adapters = vec![adapter(
            "external",
            vec![profile("observed", 100, Some(4096), false)],
            ResourceAdapterRuntimeState::Inactive,
            None,
        )];
        let scheduling = ready_scheduling(ResourceSchedulingIntent {
            strategy: ResourceSchedulingStrategy::PrimaryFirst,
            primary_adapter_id: Some("external".into()),
            commands: Vec::new(),
        });
        let policy = ResourceCoordinatorPolicy {
            scheduling: scheduling.intent.clone(),
            ..ResourceCoordinatorPolicy::default()
        };
        let controllers = BTreeSet::from(["external".to_string()]);
        let plan = compile_resource_candidate_plan(&CompileResourceCandidatePlanInput {
            state_revision: 5,
            policy: &policy,
            snapshot: &snapshot(7000),
            gpu_device_index: Some(0),
            adapters: &adapters,
            leases: &[],
            scheduling: &scheduling,
            controller_ids: &controllers,
        });

        assert_eq!(plan.state, ResourceCandidatePlanState::Degraded);
        assert!(!plan.executable);
        assert!(plan.transitions.is_empty());
        assert!(plan
            .reason_codes
            .iter()
            .any(|reason| reason == "resource_plan_no_selectable_profile"));
    }

    #[test]
    fn inactive_adapter_without_start_lifecycle_never_claims_executable_plan() {
        let mut unavailable = adapter(
            "render",
            vec![profile("configured", 100, Some(512), true)],
            ResourceAdapterRuntimeState::Inactive,
            None,
        );
        unavailable.descriptor.lifecycle_operations = vec![ResourceAdapterOperation::Observe];
        unavailable.descriptor.automatic_preemption = None;
        let adapters = vec![unavailable];
        let scheduling = ready_scheduling(ResourceSchedulingIntent {
            strategy: ResourceSchedulingStrategy::PrimaryFirst,
            primary_adapter_id: Some("render".into()),
            commands: Vec::new(),
        });
        let policy = ResourceCoordinatorPolicy {
            scheduling: scheduling.intent.clone(),
            ..ResourceCoordinatorPolicy::default()
        };
        let plan = compile_resource_candidate_plan(&CompileResourceCandidatePlanInput {
            state_revision: 6,
            policy: &policy,
            snapshot: &snapshot(7_000),
            gpu_device_index: Some(0),
            adapters: &adapters,
            leases: &[],
            scheduling: &scheduling,
            controller_ids: &BTreeSet::from(["render".to_string()]),
        });

        assert_eq!(plan.state, ResourceCandidatePlanState::Degraded);
        assert!(!plan.executable);
        assert!(plan.transitions.is_empty());
        assert!(plan
            .reason_codes
            .iter()
            .any(|reason| reason == "resource_plan_start_unavailable"));
    }
}

//! Host-owned catalog of resource-sensitive runtimes and activity observers.
//!
//! The registry describes real adapter capabilities. It does not choose a
//! profile or execute lifecycle operations; those decisions remain with the
//! Resource Coordinator and the concrete adapter implementation.

use std::collections::{BTreeMap, BTreeSet};

use oclive_kernel_types::{
    ResourceAdapterDescriptor, ResourceAdapterDiagnostic, ResourceAdapterKind,
    ResourceAdapterOperation, ResourceAdapterRegistration, ResourceAdapterRegistrationSource,
    ResourceAdapterRuntimeState, ResourceControlMode, ResourceLeaseDiagnostic, ResourceLeaseState,
    ResourceResidencyMode, ResourceResidencyPreference, ResourceSchedulingCommand,
    ResourceSchedulingIntent, ResourceSchedulingIntentDiagnostics, ResourceSchedulingIntentState,
    ResourceSchedulingStrategy,
};
use parking_lot::RwLock;

#[derive(Debug, Default)]
pub struct ResourceAdapterRegistry {
    descriptors: RwLock<BTreeMap<String, ResourceAdapterDescriptor>>,
    owners: RwLock<BTreeMap<String, (ResourceAdapterRegistrationSource, String)>>,
}

impl ResourceAdapterRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a descriptor, accepting an identical repeat as idempotent.
    ///
    /// # Errors
    ///
    /// Returns a stable validation message when identifiers, profiles, or
    /// lifecycle claims are malformed, or when the same ID is redefined.
    pub fn register(&self, descriptor: ResourceAdapterDescriptor) -> Result<(), String> {
        self.register_owned(ResourceAdapterRegistration {
            source: ResourceAdapterRegistrationSource::Builtin,
            source_id: "host".into(),
            descriptor,
        })
    }

    /// Register an owner-scoped adapter supplied by a host extension or
    /// directory-plugin bridge.
    ///
    /// Third-party adapters remain inside their provider namespace and cannot
    /// impersonate a builtin or another extension. Descriptor registration
    /// never grants lifecycle control.
    ///
    /// # Errors
    ///
    /// Returns a stable validation message for malformed ownership,
    /// namespace violations, or conflicting re-registration.
    pub fn register_owned(&self, registration: ResourceAdapterRegistration) -> Result<(), String> {
        validate_registration(&registration)?;
        let ResourceAdapterRegistration {
            source,
            source_id,
            descriptor,
        } = registration;
        let mut descriptors = self.descriptors.write();
        let mut owners = self.owners.write();
        if let Some(existing) = descriptors.get(&descriptor.adapter_id) {
            let same_owner = owners
                .get(&descriptor.adapter_id)
                .is_some_and(|owner| owner == &(source, source_id.clone()));
            return if existing == &descriptor && same_owner {
                Ok(())
            } else {
                Err(format!(
                    "resource adapter {} already registered with a different descriptor",
                    descriptor.adapter_id
                ))
            };
        }
        owners.insert(descriptor.adapter_id.clone(), (source, source_id));
        descriptors.insert(descriptor.adapter_id.clone(), descriptor);
        Ok(())
    }

    #[must_use]
    pub fn contains(&self, adapter_id: &str) -> bool {
        self.descriptors.read().contains_key(adapter_id)
    }

    #[must_use]
    pub fn descriptor(&self, adapter_id: &str) -> Option<ResourceAdapterDescriptor> {
        self.descriptors.read().get(adapter_id).cloned()
    }

    #[must_use]
    pub fn profile_is_registered(&self, adapter_id: &str, profile_id: &str) -> bool {
        self.descriptors
            .read()
            .get(adapter_id)
            .is_some_and(|descriptor| {
                descriptor
                    .profiles
                    .iter()
                    .any(|profile| profile.profile_id == profile_id)
            })
    }

    #[must_use]
    pub fn registration_owner(
        &self,
        adapter_id: &str,
    ) -> Option<(ResourceAdapterRegistrationSource, String)> {
        self.owners.read().get(adapter_id).cloned()
    }

    #[must_use]
    pub fn diagnostics(
        &self,
        leases: &BTreeMap<String, ResourceLeaseDiagnostic>,
    ) -> (Vec<ResourceAdapterDiagnostic>, Vec<String>) {
        let descriptors = self.descriptors.read();
        let owners = self.owners.read();
        let mut diagnostics = descriptors
            .values()
            .cloned()
            .map(|descriptor| {
                let owner = owners
                    .get(&descriptor.adapter_id)
                    .cloned()
                    .unwrap_or((ResourceAdapterRegistrationSource::Builtin, "host".into()));
                diagnostic_for_descriptor(descriptor, owner, leases)
            })
            .collect::<Vec<_>>();
        diagnostics
            .sort_by(|left, right| left.descriptor.adapter_id.cmp(&right.descriptor.adapter_id));

        let unregistered = leases
            .values()
            .filter(|lease| !descriptors.contains_key(&lease.adapter_id))
            .map(|lease| lease.adapter_id.clone())
            .collect::<BTreeSet<_>>();
        let reason_codes = if unregistered.is_empty() {
            Vec::new()
        } else {
            vec!["resource_adapter_unregistered".into()]
        };
        (diagnostics, reason_codes)
    }

    #[must_use]
    pub fn scheduling_diagnostics(
        &self,
        intent: &ResourceSchedulingIntent,
    ) -> ResourceSchedulingIntentDiagnostics {
        let descriptors = self.descriptors.read();
        let mut blocked_reasons = BTreeSet::new();
        let mut degraded_reasons = BTreeSet::new();

        if intent.strategy == ResourceSchedulingStrategy::PrimaryFirst
            && intent.primary_adapter_id.is_none()
        {
            blocked_reasons.insert("resource_primary_adapter_required".into());
        }
        if intent.strategy == ResourceSchedulingStrategy::Custom && intent.commands.is_empty() {
            blocked_reasons.insert("resource_custom_schedule_empty".into());
        }
        if let Some(primary_adapter_id) = intent.primary_adapter_id.as_deref() {
            if primary_adapter_id.trim().is_empty()
                || primary_adapter_id.trim() != primary_adapter_id
            {
                blocked_reasons.insert("resource_scheduling_adapter_id_invalid".into());
            } else {
                match descriptors.get(primary_adapter_id) {
                    None => {
                        blocked_reasons.insert("resource_scheduling_adapter_unregistered".into());
                    }
                    Some(descriptor)
                        if descriptor.control_mode == ResourceControlMode::ObserveOnly =>
                    {
                        degraded_reasons.insert("resource_primary_adapter_observe_only".into());
                    }
                    Some(_) => {}
                }
            }
        }

        let mut validation = SchedulingValidation::new(&descriptors);
        for command in &intent.commands {
            validation.validate_command(command);
        }
        if validation.coexist_groups.iter().any(|coexist_group| {
            validation.exclusive_groups.iter().any(|exclusive_group| {
                coexist_group
                    .iter()
                    .filter(|adapter_id| exclusive_group.contains(adapter_id))
                    .take(2)
                    .count()
                    >= 2
            })
        }) {
            validation
                .blocked_reasons
                .insert("resource_scheduling_group_conflict".into());
        }
        blocked_reasons.extend(validation.blocked_reasons);
        degraded_reasons.extend(validation.degraded_reasons);

        let (state, reason_codes) = if blocked_reasons.is_empty() {
            if degraded_reasons.is_empty() {
                (ResourceSchedulingIntentState::Ready, Vec::new())
            } else {
                (
                    ResourceSchedulingIntentState::Degraded,
                    degraded_reasons.into_iter().collect(),
                )
            }
        } else {
            blocked_reasons.extend(degraded_reasons);
            (
                ResourceSchedulingIntentState::Blocked,
                blocked_reasons.into_iter().collect(),
            )
        };
        ResourceSchedulingIntentDiagnostics {
            state,
            intent: intent.clone(),
            reason_codes,
        }
    }
}

struct SchedulingValidation<'a> {
    descriptors: &'a BTreeMap<String, ResourceAdapterDescriptor>,
    residency_by_adapter: BTreeMap<String, ResourceResidencyPreference>,
    coexist_groups: BTreeSet<Vec<String>>,
    exclusive_groups: BTreeSet<Vec<String>>,
    blocked_reasons: BTreeSet<String>,
    degraded_reasons: BTreeSet<String>,
}

impl<'a> SchedulingValidation<'a> {
    fn new(descriptors: &'a BTreeMap<String, ResourceAdapterDescriptor>) -> Self {
        Self {
            descriptors,
            residency_by_adapter: BTreeMap::new(),
            coexist_groups: BTreeSet::new(),
            exclusive_groups: BTreeSet::new(),
            blocked_reasons: BTreeSet::new(),
            degraded_reasons: BTreeSet::new(),
        }
    }

    fn validate_command(&mut self, command: &ResourceSchedulingCommand) {
        match command {
            ResourceSchedulingCommand::Require { adapter_id } => {
                self.require_registered_adapter(adapter_id);
            }
            ResourceSchedulingCommand::Residency { adapter_id, mode } => {
                let Some(descriptor) = self.require_registered_adapter(adapter_id) else {
                    return;
                };
                if self
                    .residency_by_adapter
                    .insert(adapter_id.clone(), *mode)
                    .is_some_and(|existing| existing != *mode)
                {
                    self.blocked_reasons
                        .insert("resource_scheduling_residency_conflict".into());
                }
                let residency_mode = match mode {
                    ResourceResidencyPreference::Resident => ResourceResidencyMode::Resident,
                    ResourceResidencyPreference::OnDemand => ResourceResidencyMode::OnDemand,
                };
                if !descriptor.residency_modes.contains(&residency_mode) {
                    self.blocked_reasons
                        .insert("resource_scheduling_residency_unsupported".into());
                }
            }
            ResourceSchedulingCommand::Coexist { adapter_ids } => {
                if let Some(group) = self.validated_adapter_group(adapter_ids) {
                    self.coexist_groups.insert(group);
                }
            }
            ResourceSchedulingCommand::Exclusive { adapter_ids } => {
                if let Some(group) = self.validated_adapter_group(adapter_ids) {
                    if group.iter().any(|adapter_id| {
                        self.descriptors.get(adapter_id).is_some_and(|descriptor| {
                            descriptor.control_mode == ResourceControlMode::ObserveOnly
                        })
                    }) {
                        self.blocked_reasons
                            .insert("resource_scheduling_control_unavailable".into());
                    }
                    self.exclusive_groups.insert(group);
                }
            }
            ResourceSchedulingCommand::YieldThenRun {
                yielding_adapter_id,
                target_adapter_id,
            } => {
                self.validate_yield_then_run(yielding_adapter_id, target_adapter_id);
            }
            ResourceSchedulingCommand::Fallback {
                adapter_id,
                profile_ids,
            } => {
                self.validate_fallback(adapter_id, profile_ids);
            }
        }
    }

    fn validate_yield_then_run(&mut self, yielding_adapter_id: &str, target_adapter_id: &str) {
        if yielding_adapter_id == target_adapter_id {
            self.blocked_reasons
                .insert("resource_scheduling_self_transition".into());
            return;
        }
        let yielding = self.require_registered_adapter(yielding_adapter_id);
        let target = self.require_registered_adapter(target_adapter_id);
        if let Some(descriptor) = yielding {
            let can_yield = descriptor.control_mode == ResourceControlMode::Managed
                && descriptor.lifecycle_operations.iter().any(|operation| {
                    matches!(
                        operation,
                        ResourceAdapterOperation::Suspend | ResourceAdapterOperation::Unload
                    )
                });
            if !can_yield {
                self.blocked_reasons
                    .insert("resource_scheduling_yield_unsupported".into());
            }
        }
        if let Some(descriptor) = target {
            let can_run = descriptor.control_mode == ResourceControlMode::Managed
                && descriptor.lifecycle_operations.iter().any(|operation| {
                    matches!(
                        operation,
                        ResourceAdapterOperation::Start | ResourceAdapterOperation::Resume
                    )
                });
            if !can_run {
                self.blocked_reasons
                    .insert("resource_scheduling_start_unsupported".into());
            }
        }
    }

    fn validate_fallback(&mut self, adapter_id: &str, profile_ids: &[String]) {
        let Some(descriptor) = self.require_registered_adapter(adapter_id) else {
            return;
        };
        if profile_ids.is_empty() {
            self.blocked_reasons
                .insert("resource_scheduling_fallback_empty".into());
            return;
        }
        let mut seen = BTreeSet::new();
        for profile_id in profile_ids {
            if profile_id.trim().is_empty()
                || profile_id.trim() != profile_id
                || !seen.insert(profile_id)
            {
                self.blocked_reasons
                    .insert("resource_scheduling_profile_invalid".into());
                continue;
            }
            match descriptor
                .profiles
                .iter()
                .find(|profile| profile.profile_id == *profile_id)
            {
                None => {
                    self.blocked_reasons
                        .insert("resource_profile_unregistered".into());
                }
                Some(profile) if !profile.coordinator_selectable => {
                    self.degraded_reasons
                        .insert("resource_profile_not_coordinator_selectable".into());
                }
                Some(_) => {}
            }
        }
    }

    fn require_registered_adapter(
        &mut self,
        adapter_id: &str,
    ) -> Option<&'a ResourceAdapterDescriptor> {
        if adapter_id.trim().is_empty() || adapter_id.trim() != adapter_id {
            self.blocked_reasons
                .insert("resource_scheduling_adapter_id_invalid".into());
            return None;
        }
        let descriptor = self.descriptors.get(adapter_id);
        if descriptor.is_none() {
            self.blocked_reasons
                .insert("resource_scheduling_adapter_unregistered".into());
        }
        descriptor
    }

    fn validated_adapter_group(&mut self, adapter_ids: &[String]) -> Option<Vec<String>> {
        if adapter_ids.len() < 2 {
            self.blocked_reasons
                .insert("resource_scheduling_group_too_small".into());
            return None;
        }
        let mut group = BTreeSet::new();
        for adapter_id in adapter_ids {
            self.require_registered_adapter(adapter_id)?;
            if !group.insert(adapter_id.clone()) {
                self.blocked_reasons
                    .insert("resource_scheduling_group_duplicate".into());
                return None;
            }
        }
        Some(group.into_iter().collect())
    }
}

fn validate_registration(registration: &ResourceAdapterRegistration) -> Result<(), String> {
    let source_id = registration.source_id.as_str();
    if !canonical_owner_id(source_id) {
        return Err("resource adapter registration source id is invalid".into());
    }
    let descriptor = &registration.descriptor;
    validate_descriptor(descriptor)?;
    if registration.source != ResourceAdapterRegistrationSource::Builtin {
        let namespaced = descriptor.adapter_id == source_id
            || descriptor
                .adapter_id
                .strip_prefix(source_id)
                .is_some_and(|suffix| suffix.starts_with('.'));
        if !namespaced || descriptor.adapter_id.starts_with("builtin.") {
            return Err(format!(
                "third-party resource adapter {} is outside owner namespace {source_id}",
                descriptor.adapter_id
            ));
        }
        if descriptor.provider_id.as_deref() != Some(source_id) {
            return Err(format!(
                "third-party resource adapter {} provider id must match owner {source_id}",
                descriptor.adapter_id
            ));
        }
    }
    Ok(())
}

fn canonical_owner_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.trim() == value
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
        && value
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphanumeric)
        && value
            .as_bytes()
            .last()
            .is_some_and(u8::is_ascii_alphanumeric)
        && !value.contains("..")
}

fn validate_descriptor(descriptor: &ResourceAdapterDescriptor) -> Result<(), String> {
    let adapter_id = descriptor.adapter_id.as_str();
    if !canonical_owner_id(adapter_id) {
        return Err("resource adapter id is invalid".into());
    }
    if let Some(provider_id) = descriptor.provider_id.as_deref() {
        if !canonical_owner_id(provider_id) {
            return Err(format!(
                "resource adapter {adapter_id} provider id is invalid"
            ));
        }
    }
    if descriptor.kind == ResourceAdapterKind::Runtime && descriptor.profiles.is_empty() {
        return Err(format!(
            "resource runtime adapter {adapter_id} must declare at least one profile"
        ));
    }
    if descriptor.control_mode == ResourceControlMode::ObserveOnly
        && descriptor.lifecycle_operations.iter().any(|operation| {
            !matches!(
                operation,
                oclive_kernel_types::ResourceAdapterOperation::Observe
            )
        })
    {
        return Err(format!(
            "observe-only resource adapter {adapter_id} cannot claim managed lifecycle operations"
        ));
    }
    if let Some(operation) = descriptor.automatic_preemption {
        if descriptor.control_mode != ResourceControlMode::Managed
            || !matches!(
                operation,
                ResourceAdapterOperation::Suspend | ResourceAdapterOperation::Unload
            )
            || !descriptor.lifecycle_operations.contains(&operation)
        {
            return Err(format!(
                "resource adapter {adapter_id} automatic preemption is not controllable"
            ));
        }
        let Some(restore) = (match operation {
            ResourceAdapterOperation::Suspend => Some(ResourceAdapterOperation::Resume),
            ResourceAdapterOperation::Unload => Some(ResourceAdapterOperation::Start),
            ResourceAdapterOperation::Observe
            | ResourceAdapterOperation::Start
            | ResourceAdapterOperation::Resume
            | ResourceAdapterOperation::Release => None,
        }) else {
            return Err(format!(
                "resource adapter {adapter_id} automatic preemption is not reversible"
            ));
        };
        if !descriptor.lifecycle_operations.contains(&restore) {
            return Err(format!(
                "resource adapter {adapter_id} automatic preemption has no restore operation"
            ));
        }
    }

    let mut profile_ids = BTreeSet::new();
    for profile in &descriptor.profiles {
        let profile_id = profile.profile_id.as_str();
        if !canonical_owner_id(profile_id) {
            return Err(format!(
                "resource adapter {adapter_id} profile id is invalid"
            ));
        }
        if !profile_ids.insert(profile_id) {
            return Err(format!(
                "resource adapter {adapter_id} repeats profile {profile_id}"
            ));
        }
        if descriptor.control_mode == ResourceControlMode::ObserveOnly
            && profile.coordinator_selectable
        {
            return Err(format!(
                "observe-only resource adapter {adapter_id} profile {profile_id} cannot be coordinator-selectable"
            ));
        }
    }
    Ok(())
}

fn diagnostic_for_descriptor(
    descriptor: ResourceAdapterDescriptor,
    owner: (ResourceAdapterRegistrationSource, String),
    leases: &BTreeMap<String, ResourceLeaseDiagnostic>,
) -> ResourceAdapterDiagnostic {
    let matching = leases
        .values()
        .filter(|lease| lease.adapter_id == descriptor.adapter_id)
        .collect::<Vec<_>>();
    let has_active = matching
        .iter()
        .any(|lease| lease.state == ResourceLeaseState::Active);
    let has_reserved = matching
        .iter()
        .any(|lease| lease.state == ResourceLeaseState::Reserved);
    let runtime_state = if has_active {
        ResourceAdapterRuntimeState::Active
    } else if has_reserved {
        ResourceAdapterRuntimeState::Reserved
    } else if descriptor.control_mode == ResourceControlMode::ObserveOnly {
        ResourceAdapterRuntimeState::Unknown
    } else {
        ResourceAdapterRuntimeState::Inactive
    };
    let current_profile_id = matching
        .iter()
        .find(|lease| lease.state == ResourceLeaseState::Active)
        .or_else(|| matching.first())
        .and_then(|lease| lease.profile_id.clone());
    let registered_profiles = descriptor
        .profiles
        .iter()
        .map(|profile| profile.profile_id.as_str())
        .collect::<BTreeSet<_>>();
    let mut reason_codes = matching
        .iter()
        .flat_map(|lease| lease.reason_codes.iter().cloned())
        .collect::<BTreeSet<_>>();
    if current_profile_id
        .as_deref()
        .filter(|profile_id| !registered_profiles.contains(profile_id))
        .is_some()
    {
        reason_codes.insert("resource_profile_unregistered".into());
    }

    ResourceAdapterDiagnostic {
        descriptor,
        registration_source: owner.0,
        registration_source_id: owner.1,
        runtime_state,
        current_profile_id,
        lease_ids: matching
            .into_iter()
            .map(|lease| lease.lease_id.clone())
            .collect(),
        reason_codes: reason_codes.into_iter().collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oclive_kernel_types::{
        ResourceAdapterDomain, ResourceAdapterOperation, ResourceExecutionTarget,
        ResourceOperatingProfile, ResourcePriority, ResourceResidencyMode,
        ResourceResidencyPreference, ResourceSchedulingCommand, ResourceSchedulingIntent,
        ResourceSchedulingIntentState, ResourceSchedulingStrategy,
    };

    fn managed_descriptor(adapter_id: &str) -> ResourceAdapterDescriptor {
        ResourceAdapterDescriptor {
            adapter_id: adapter_id.into(),
            kind: ResourceAdapterKind::Runtime,
            domain: ResourceAdapterDomain::Llm,
            provider_id: Some("builtin.test".into()),
            control_mode: ResourceControlMode::Managed,
            profiles: vec![ResourceOperatingProfile {
                profile_id: "configured".into(),
                quality_rank: 100,
                execution_target: ResourceExecutionTarget::Gpu,
                estimated_reservation_mib: None,
                estimated_ram_mib: None,
                estimated_cpu_threads: None,
                requires_restart: true,
                coordinator_selectable: false,
            }],
            lifecycle_operations: vec![
                ResourceAdapterOperation::Start,
                ResourceAdapterOperation::Resume,
                ResourceAdapterOperation::Suspend,
                ResourceAdapterOperation::Unload,
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

    #[test]
    fn identical_registration_is_idempotent_but_conflicts_are_rejected() {
        let registry = ResourceAdapterRegistry::new();
        let descriptor = managed_descriptor("builtin.test");
        registry.register(descriptor.clone()).unwrap();
        registry.register(descriptor).unwrap();

        let mut conflicting = managed_descriptor("builtin.test");
        conflicting.provider_id = Some("different".into());
        assert!(registry.register(conflicting).is_err());
    }

    #[test]
    fn third_party_render_registration_is_owner_scoped_and_traceable() {
        let registry = ResourceAdapterRegistry::new();
        let mut descriptor = managed_descriptor("com.example.live2d.render");
        descriptor.domain = ResourceAdapterDomain::Render;
        descriptor.provider_id = Some("com.example.live2d".into());
        let registration = ResourceAdapterRegistration {
            source: ResourceAdapterRegistrationSource::HostExtension,
            source_id: "com.example.live2d".into(),
            descriptor: descriptor.clone(),
        };
        registry.register_owned(registration.clone()).unwrap();
        registry.register_owned(registration).unwrap();

        let (diagnostics, reasons) = registry.diagnostics(&BTreeMap::new());
        assert!(reasons.is_empty());
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].registration_source,
            ResourceAdapterRegistrationSource::HostExtension
        );
        assert_eq!(diagnostics[0].registration_source_id, "com.example.live2d");

        let mut impersonating = descriptor.clone();
        impersonating.adapter_id = "builtin.render.live2d".into();
        assert!(registry
            .register_owned(ResourceAdapterRegistration {
                source: ResourceAdapterRegistrationSource::HostExtension,
                source_id: "com.example.live2d".into(),
                descriptor: impersonating,
            })
            .unwrap_err()
            .contains("outside owner namespace"));

        descriptor.provider_id = Some("com.other".into());
        assert!(registry
            .register_owned(ResourceAdapterRegistration {
                source: ResourceAdapterRegistrationSource::DirectoryPlugin,
                source_id: "com.example.live2d".into(),
                descriptor,
            })
            .unwrap_err()
            .contains("provider id must match owner"));
    }

    #[test]
    fn automatic_preemption_requires_a_reversible_managed_operation() {
        let registry = ResourceAdapterRegistry::new();
        let mut descriptor = managed_descriptor("builtin.invalid");
        descriptor
            .lifecycle_operations
            .retain(|operation| *operation != ResourceAdapterOperation::Resume);
        assert!(registry
            .register(descriptor)
            .unwrap_err()
            .contains("has no restore operation"));

        let mut non_preemptive = managed_descriptor("builtin.non_preemptive");
        non_preemptive.automatic_preemption = Some(ResourceAdapterOperation::Start);
        assert!(registry
            .register(non_preemptive)
            .unwrap_err()
            .contains("automatic preemption is not controllable"));
    }

    #[test]
    fn identifiers_must_be_canonical_before_registration() {
        let registry = ResourceAdapterRegistry::new();
        assert!(registry.register(managed_descriptor(" padded")).is_err());

        let mut padded_provider = managed_descriptor("builtin.provider");
        padded_provider.provider_id = Some(" builtin.test".into());
        assert!(registry.register(padded_provider).is_err());

        let mut padded_profile = managed_descriptor("builtin.profile");
        padded_profile.profiles[0].profile_id = "full ".into();
        assert!(registry.register(padded_profile).is_err());

        assert!(registry
            .register(managed_descriptor("builtin.bad\nid"))
            .is_err());
    }

    #[test]
    fn diagnostics_join_registered_descriptor_with_active_lease() {
        let registry = ResourceAdapterRegistry::new();
        registry
            .register(managed_descriptor("builtin.test"))
            .unwrap();
        let mut leases = BTreeMap::new();
        leases.insert(
            "lease-1".into(),
            ResourceLeaseDiagnostic {
                lease_id: "lease-1".into(),
                adapter_id: "builtin.test".into(),
                workload_id: "runtime".into(),
                profile_id: Some("configured".into()),
                gpu_device_index: Some(0),
                reservation_mib: 1024,
                actual_mib: 1024,
                ram_reservation_mib: 0,
                actual_ram_mib: 0,
                cpu_thread_reservation: 0,
                actual_cpu_threads: 0,
                priority: ResourcePriority::Resident,
                control_mode: ResourceControlMode::Managed,
                state: ResourceLeaseState::Active,
                granted_at_ms: 1,
                expires_at_ms: None,
                reason_codes: vec!["resource_release_unconfirmed".into()],
            },
        );

        let (diagnostics, reasons) = registry.diagnostics(&leases);
        assert!(reasons.is_empty());
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].runtime_state,
            ResourceAdapterRuntimeState::Active
        );
        assert_eq!(
            diagnostics[0].current_profile_id.as_deref(),
            Some("configured")
        );
        assert_eq!(
            diagnostics[0].reason_codes,
            vec!["resource_release_unconfirmed"]
        );
    }

    #[test]
    fn unregistered_lease_is_visible_as_registry_drift() {
        let registry = ResourceAdapterRegistry::new();
        let mut leases = BTreeMap::new();
        leases.insert(
            "lease-1".into(),
            ResourceLeaseDiagnostic {
                lease_id: "lease-1".into(),
                adapter_id: "unknown".into(),
                workload_id: "runtime".into(),
                profile_id: None,
                gpu_device_index: None,
                reservation_mib: 0,
                actual_mib: 0,
                ram_reservation_mib: 0,
                actual_ram_mib: 0,
                cpu_thread_reservation: 0,
                actual_cpu_threads: 0,
                priority: ResourcePriority::ForegroundInteractive,
                control_mode: ResourceControlMode::ObserveOnly,
                state: ResourceLeaseState::Active,
                granted_at_ms: 1,
                expires_at_ms: Some(2),
                reason_codes: Vec::new(),
            },
        );

        let (_, reasons) = registry.diagnostics(&leases);
        assert_eq!(reasons, vec!["resource_adapter_unregistered"]);
    }

    #[test]
    fn finite_scheduling_commands_validate_against_adapter_facts() {
        let registry = ResourceAdapterRegistry::new();
        registry
            .register(managed_descriptor("builtin.llm"))
            .unwrap();
        registry
            .register(managed_descriptor("builtin.voice"))
            .unwrap();
        let diagnostics = registry.scheduling_diagnostics(&ResourceSchedulingIntent {
            strategy: ResourceSchedulingStrategy::PrimaryFirst,
            primary_adapter_id: Some("builtin.llm".into()),
            commands: vec![
                ResourceSchedulingCommand::Require {
                    adapter_id: "builtin.llm".into(),
                },
                ResourceSchedulingCommand::Residency {
                    adapter_id: "builtin.voice".into(),
                    mode: ResourceResidencyPreference::OnDemand,
                },
                ResourceSchedulingCommand::YieldThenRun {
                    yielding_adapter_id: "builtin.llm".into(),
                    target_adapter_id: "builtin.voice".into(),
                },
            ],
        });
        assert_eq!(diagnostics.state, ResourceSchedulingIntentState::Ready);
        assert!(diagnostics.reason_codes.is_empty());
    }

    #[test]
    fn scheduling_diagnostics_reject_conflicts_and_report_unselectable_profiles() {
        let registry = ResourceAdapterRegistry::new();
        registry
            .register(managed_descriptor("builtin.llm"))
            .unwrap();
        registry
            .register(managed_descriptor("builtin.voice"))
            .unwrap();
        registry
            .register(managed_descriptor("builtin.render"))
            .unwrap();
        let diagnostics = registry.scheduling_diagnostics(&ResourceSchedulingIntent {
            strategy: ResourceSchedulingStrategy::Custom,
            primary_adapter_id: None,
            commands: vec![
                ResourceSchedulingCommand::Coexist {
                    adapter_ids: vec!["builtin.llm".into(), "builtin.voice".into()],
                },
                ResourceSchedulingCommand::Exclusive {
                    adapter_ids: vec![
                        "builtin.voice".into(),
                        "builtin.render".into(),
                        "builtin.llm".into(),
                    ],
                },
                ResourceSchedulingCommand::Fallback {
                    adapter_id: "builtin.llm".into(),
                    profile_ids: vec!["configured".into()],
                },
            ],
        });
        assert_eq!(diagnostics.state, ResourceSchedulingIntentState::Blocked);
        assert_eq!(
            diagnostics.reason_codes,
            vec![
                "resource_profile_not_coordinator_selectable",
                "resource_scheduling_group_conflict",
            ]
        );
    }

    #[test]
    fn primary_adapter_id_preserves_invalid_input_for_traceable_diagnostics() {
        let registry = ResourceAdapterRegistry::new();
        let diagnostics = registry.scheduling_diagnostics(&ResourceSchedulingIntent {
            strategy: ResourceSchedulingStrategy::PrimaryFirst,
            primary_adapter_id: Some(" builtin.llm".into()),
            commands: Vec::new(),
        });
        assert_eq!(diagnostics.state, ResourceSchedulingIntentState::Blocked);
        assert_eq!(
            diagnostics.reason_codes,
            vec!["resource_scheduling_adapter_id_invalid"]
        );
    }
}

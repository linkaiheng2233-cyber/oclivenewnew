//! Host-owned catalog of resource-sensitive runtimes and activity observers.
//!
//! The registry describes real adapter capabilities. It does not choose a
//! profile or execute lifecycle operations; those decisions remain with the
//! Resource Coordinator and the concrete adapter implementation.

use std::collections::{BTreeMap, BTreeSet};

use oclive_kernel_types::{
    ResourceAdapterDescriptor, ResourceAdapterDiagnostic, ResourceAdapterKind,
    ResourceAdapterRuntimeState, ResourceControlMode, ResourceLeaseDiagnostic, ResourceLeaseState,
};
use parking_lot::RwLock;

#[derive(Debug, Default)]
pub struct ResourceAdapterRegistry {
    descriptors: RwLock<BTreeMap<String, ResourceAdapterDescriptor>>,
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
        validate_descriptor(&descriptor)?;
        let mut descriptors = self.descriptors.write();
        if let Some(existing) = descriptors.get(&descriptor.adapter_id) {
            return if existing == &descriptor {
                Ok(())
            } else {
                Err(format!(
                    "resource adapter {} already registered with a different descriptor",
                    descriptor.adapter_id
                ))
            };
        }
        descriptors.insert(descriptor.adapter_id.clone(), descriptor);
        Ok(())
    }

    #[must_use]
    pub fn contains(&self, adapter_id: &str) -> bool {
        self.descriptors.read().contains_key(adapter_id)
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
    pub fn diagnostics(
        &self,
        leases: &BTreeMap<String, ResourceLeaseDiagnostic>,
    ) -> (Vec<ResourceAdapterDiagnostic>, Vec<String>) {
        let descriptors = self.descriptors.read();
        let mut diagnostics = descriptors
            .values()
            .cloned()
            .map(|descriptor| diagnostic_for_descriptor(descriptor, leases))
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
}

fn validate_descriptor(descriptor: &ResourceAdapterDescriptor) -> Result<(), String> {
    let adapter_id = descriptor.adapter_id.trim();
    if adapter_id.is_empty() {
        return Err("resource adapter id must not be empty".into());
    }
    if adapter_id != descriptor.adapter_id {
        return Err("resource adapter id must not contain surrounding whitespace".into());
    }
    if let Some(provider_id) = descriptor.provider_id.as_deref() {
        if provider_id.trim().is_empty() {
            return Err(format!(
                "resource adapter {adapter_id} provider id must not be empty"
            ));
        }
        if provider_id.trim() != provider_id {
            return Err(format!(
                "resource adapter {adapter_id} provider id must not contain surrounding whitespace"
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

    let mut profile_ids = BTreeSet::new();
    for profile in &descriptor.profiles {
        let profile_id = profile.profile_id.trim();
        if profile_id.is_empty() {
            return Err(format!(
                "resource adapter {adapter_id} profile id must not be empty"
            ));
        }
        if profile_id != profile.profile_id {
            return Err(format!(
                "resource adapter {adapter_id} profile id must not contain surrounding whitespace"
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
    let reason_codes = current_profile_id
        .as_deref()
        .filter(|profile_id| !registered_profiles.contains(profile_id))
        .map_or_else(Vec::new, |_| vec!["resource_profile_unregistered".into()]);

    ResourceAdapterDiagnostic {
        descriptor,
        runtime_state,
        current_profile_id,
        lease_ids: matching
            .into_iter()
            .map(|lease| lease.lease_id.clone())
            .collect(),
        reason_codes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oclive_kernel_types::{
        ResourceAdapterDomain, ResourceAdapterOperation, ResourceExecutionTarget,
        ResourceOperatingProfile, ResourcePriority, ResourceResidencyMode,
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
                requires_restart: true,
                coordinator_selectable: false,
            }],
            lifecycle_operations: vec![
                ResourceAdapterOperation::Start,
                ResourceAdapterOperation::Unload,
            ],
            residency_modes: vec![
                ResourceResidencyMode::Resident,
                ResourceResidencyMode::Unloaded,
            ],
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
    fn identifiers_must_be_canonical_before_registration() {
        let registry = ResourceAdapterRegistry::new();
        assert!(registry.register(managed_descriptor(" padded")).is_err());

        let mut padded_provider = managed_descriptor("builtin.provider");
        padded_provider.provider_id = Some(" builtin.test".into());
        assert!(registry.register(padded_provider).is_err());

        let mut padded_profile = managed_descriptor("builtin.profile");
        padded_profile.profiles[0].profile_id = "full ".into();
        assert!(registry.register(padded_profile).is_err());
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
                priority: ResourcePriority::Resident,
                control_mode: ResourceControlMode::Managed,
                state: ResourceLeaseState::Active,
                granted_at_ms: 1,
                expires_at_ms: None,
                reason_codes: Vec::new(),
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
}

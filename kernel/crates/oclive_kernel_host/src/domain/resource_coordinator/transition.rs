//! Resource transitions: plans, execution, rollback, and preemption candidates.

use super::{
    diagnostics_from_state, now_epoch_ms, prune_expired, restore_operation_for,
    transition_releases_residency, AutomaticPreemptionCandidate, CoordinatorState,
    ResourceCoordinator,
};

use oclive_kernel_contracts::ResourceAdapterController;
use oclive_kernel_types::{
    AppError, ResourceAdapterOperation, ResourceAdapterTransitionRequest,
    ResourceAdapterTransitionResponse, ResourceAdmissionRequest, ResourceCandidatePlan,
    ResourceCandidateTransition, ResourceControlMode, ResourceCoordinationDiagnostics,
    ResourceLeaseState, RESOURCE_COORDINATION_SCHEMA_VERSION,
};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::Ordering;
use std::sync::Arc;

impl ResourceCoordinator {
    /// Execute one lifecycle operation through the adapter's authoritative
    /// controller after descriptor, caller, profile, revision, and lock checks.
    ///
    /// # Errors
    ///
    /// Returns stable invalid-parameter or unavailable errors when the request
    /// is stale, unsupported, uncontrolled, or rejected by the runtime.
    pub async fn transition_adapter(
        &self,
        request: &ResourceAdapterTransitionRequest,
    ) -> Result<ResourceAdapterTransitionResponse, AppError> {
        let controller = self.validate_transition(
            &request.adapter_id,
            request.operation,
            request.profile_id.as_deref(),
            Some(&request.requested_by_adapter_id),
            false,
        )?;
        // Only the lifecycle target is a single-writer resource. Callers may
        // already hold their own adapter lock across admission and use (the
        // bundled voice path does), so recursively locking the requester here
        // would deadlock automatic preemption.
        let _guards = self
            .lock_adapter_operations([request.adapter_id.as_str()])
            .await;
        self.ensure_expected_revision(request.expected_revision)?;
        let outcome = controller
            .transition(
                request.operation,
                request.profile_id.as_deref(),
                request.reason.as_deref(),
            )
            .await?;
        let lease_changed = if transition_releases_residency(request.operation) {
            self.release_adapter(&request.adapter_id) > 0
        } else {
            false
        };
        if !outcome.already_in_state && !lease_changed {
            self.bump_revision();
        }
        Ok(ResourceAdapterTransitionResponse {
            schema_version: RESOURCE_COORDINATION_SCHEMA_VERSION,
            adapter_id: request.adapter_id.clone(),
            operation: request.operation,
            requested_by_adapter_id: request.requested_by_adapter_id.clone(),
            already_in_state: outcome.already_in_state,
            recovery_scheduled: outcome.recovery_scheduled,
            state_revision: self.state_revision(),
        })
    }

    /// Execute a previously compiled candidate plan as one serialized batch.
    ///
    /// The method is intentionally not exposed as an HTTP command yet. It is
    /// the generic host foundation used by controlled callers after a plan has
    /// been reviewed. Completed steps are rolled back in reverse order when a
    /// later step fails; rollback failure is reported and never treated as
    /// confirmed release.
    ///
    /// # Errors
    ///
    /// Rejects stale, blocked, non-executable, unsupported, or failed plans.
    pub async fn execute_candidate_plan(
        &self,
        plan: &ResourceCandidatePlan,
    ) -> Result<Vec<ResourceAdapterTransitionResponse>, AppError> {
        if plan.state == oclive_kernel_types::ResourceCandidatePlanState::Blocked {
            return Err(AppError::InvalidParameter("resource_plan_blocked".into()));
        }
        if !plan.executable {
            return Err(AppError::InvalidParameter(
                "resource_plan_not_executable".into(),
            ));
        }
        let current_plan = self.diagnostics_snapshot().candidate_plan;
        if current_plan.plan_id != plan.plan_id
            || current_plan.compiled_from_revision != plan.compiled_from_revision
            || current_plan.selections != plan.selections
            || current_plan.transitions != plan.transitions
        {
            return Err(AppError::InvalidParameter(
                "resource_plan_candidate_mismatch".into(),
            ));
        }
        let adapter_ids = plan
            .transitions
            .iter()
            .map(|transition| transition.adapter_id.as_str());
        let _guards = self.lock_adapter_operations(adapter_ids).await;
        self.ensure_expected_revision(Some(plan.compiled_from_revision))?;

        let prepared = plan
            .transitions
            .iter()
            .map(|transition| {
                self.validate_transition(
                    &transition.adapter_id,
                    transition.operation,
                    transition.profile_id.as_deref(),
                    None,
                    false,
                )
                .map(|controller| (transition, controller))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut completed: Vec<(&ResourceCandidateTransition, bool)> = Vec::new();
        let mut responses = Vec::with_capacity(plan.transitions.len());
        for (transition, controller) in prepared {
            let reason =
                (!transition.reason_codes.is_empty()).then(|| transition.reason_codes.join(","));
            match controller
                .transition(
                    transition.operation,
                    transition.profile_id.as_deref(),
                    reason.as_deref(),
                )
                .await
            {
                Ok(outcome) => {
                    let lease_changed = if transition_releases_residency(transition.operation) {
                        self.release_adapter(&transition.adapter_id) > 0
                    } else {
                        false
                    };
                    if !outcome.already_in_state && !lease_changed {
                        self.bump_revision();
                    }
                    completed.push((transition, !outcome.already_in_state));
                    responses.push(ResourceAdapterTransitionResponse {
                        schema_version: RESOURCE_COORDINATION_SCHEMA_VERSION,
                        adapter_id: transition.adapter_id.clone(),
                        operation: transition.operation,
                        requested_by_adapter_id: transition
                            .requested_by_adapter_id
                            .clone()
                            .unwrap_or_else(|| "host.resource_coordinator".into()),
                        already_in_state: outcome.already_in_state,
                        recovery_scheduled: outcome.recovery_scheduled,
                        state_revision: self.state_revision(),
                    });
                }
                Err(error) => {
                    let rollback_errors = self.rollback_completed(&completed).await;
                    let rollback_detail = if rollback_errors.is_empty() {
                        "rollback_confirmed".to_string()
                    } else {
                        format!("rollback_failed:{}", rollback_errors.join("|"))
                    };
                    return Err(AppError::RemoteServiceUnavailable(format!(
                        "resource_plan_transition_failed:{}:{error};{rollback_detail}",
                        transition.adapter_id
                    )));
                }
            }
        }
        Ok(responses)
    }

    #[must_use]
    pub fn diagnostics_snapshot(&self) -> ResourceCoordinationDiagnostics {
        let now_ms = now_epoch_ms();
        let mut state = self.state.lock();
        if prune_expired(&mut state.leases, now_ms) {
            self.bump_revision();
        }
        self.diagnostics_from_state(&state)
    }

    async fn rollback_completed(
        &self,
        completed: &[(&ResourceCandidateTransition, bool)],
    ) -> Vec<String> {
        let mut errors = Vec::new();
        for (transition, changed_state) in completed.iter().rev() {
            if !changed_state {
                continue;
            }
            let Some(rollback_operation) = transition.rollback_operation else {
                errors.push(format!(
                    "{}:resource_plan_rollback_unavailable",
                    transition.adapter_id
                ));
                continue;
            };
            let controller = match self.validate_transition(
                &transition.adapter_id,
                rollback_operation,
                transition.rollback_profile_id.as_deref(),
                None,
                true,
            ) {
                Ok(controller) => controller,
                Err(error) => {
                    errors.push(format!("{}:{error}", transition.adapter_id));
                    continue;
                }
            };
            if let Err(error) = controller
                .transition(
                    rollback_operation,
                    transition.rollback_profile_id.as_deref(),
                    Some("resource plan rollback"),
                )
                .await
            {
                errors.push(format!("{}:{error}", transition.adapter_id));
            } else {
                let lease_changed = if transition_releases_residency(rollback_operation) {
                    self.release_adapter(&transition.adapter_id) > 0
                } else {
                    false
                };
                if !lease_changed {
                    self.bump_revision();
                }
            }
        }
        errors
    }

    fn validate_transition(
        &self,
        adapter_id: &str,
        operation: ResourceAdapterOperation,
        profile_id: Option<&str>,
        requested_by_adapter_id: Option<&str>,
        allow_nonselectable_profile: bool,
    ) -> Result<Arc<dyn ResourceAdapterController>, AppError> {
        if adapter_id.trim().is_empty() || adapter_id.trim() != adapter_id {
            return Err(AppError::InvalidParameter(
                "resource_transition_adapter_id_invalid".into(),
            ));
        }
        if let Some(requested_by) = requested_by_adapter_id {
            if requested_by == adapter_id {
                return Err(AppError::InvalidParameter(
                    "resource_transition_self_request".into(),
                ));
            }
            if !self.adapter_registry.contains(requested_by) {
                return Err(AppError::InvalidParameter(
                    "resource_transition_requester_unregistered".into(),
                ));
            }
        }
        let descriptor = self
            .adapter_registry
            .descriptor(adapter_id)
            .ok_or_else(|| {
                AppError::InvalidParameter("resource_transition_adapter_unregistered".into())
            })?;
        if descriptor.control_mode != ResourceControlMode::Managed {
            return Err(AppError::InvalidParameter(
                "resource_transition_control_unavailable".into(),
            ));
        }
        if !descriptor.lifecycle_operations.contains(&operation) {
            return Err(AppError::InvalidParameter(
                "resource_transition_operation_unsupported".into(),
            ));
        }
        if let Some(profile_id) = profile_id {
            let profile = descriptor
                .profiles
                .iter()
                .find(|profile| profile.profile_id == profile_id)
                .ok_or_else(|| {
                    AppError::InvalidParameter("resource_profile_unregistered".into())
                })?;
            if !allow_nonselectable_profile
                && matches!(
                    operation,
                    ResourceAdapterOperation::Start | ResourceAdapterOperation::Resume
                )
                && !profile.coordinator_selectable
            {
                return Err(AppError::InvalidParameter(
                    "resource_profile_not_coordinator_selectable".into(),
                ));
            }
        }
        if let Some(requested_by) = requested_by_adapter_id {
            let authorized = self
                .transition_grants
                .read()
                .get(&(requested_by.to_string(), adapter_id.to_string()))
                .is_some_and(|operations| operations.contains(&operation));
            if !authorized {
                return Err(AppError::InvalidParameter(
                    "resource_transition_not_authorized".into(),
                ));
            }
        }
        self.adapter_controllers
            .read()
            .get(adapter_id)
            .cloned()
            .ok_or_else(|| {
                AppError::RemoteServiceUnavailable(
                    "resource_transition_controller_unavailable".into(),
                )
            })
    }

    fn ensure_expected_revision(&self, expected: Option<u64>) -> Result<(), AppError> {
        if expected.is_some_and(|expected| expected != self.state_revision()) {
            return Err(AppError::RemoteServiceUnavailable(
                "resource_plan_stale_revision".into(),
            ));
        }
        Ok(())
    }

    async fn lock_adapter_operations<'a>(
        &self,
        adapter_ids: impl IntoIterator<Item = &'a str>,
    ) -> Vec<tokio::sync::OwnedMutexGuard<()>> {
        let ids = adapter_ids
            .into_iter()
            .filter(|adapter_id| !adapter_id.trim().is_empty())
            .map(str::to_string)
            .collect::<BTreeSet<_>>();
        let mut guards = Vec::with_capacity(ids.len());
        for adapter_id in ids {
            guards.push(self.lock_adapter_operation(&adapter_id).await);
        }
        guards
    }

    fn controller_ids(&self) -> BTreeSet<String> {
        self.adapter_controllers.read().keys().cloned().collect()
    }

    pub(super) fn preemption_candidates(
        &self,
        request: &ResourceAdmissionRequest,
    ) -> Vec<AutomaticPreemptionCandidate> {
        if !self.adapter_registry.contains(&request.adapter_id) {
            return Vec::new();
        }
        let now_ms = now_epoch_ms();
        let mut state = self.state.lock();
        if prune_expired(&mut state.leases, now_ms) {
            self.bump_revision();
        }
        if state.leases.values().any(|lease| {
            lease.state == ResourceLeaseState::Active
                && lease.adapter_id != request.adapter_id
                && lease.priority >= request.priority
        }) {
            return Vec::new();
        }
        let grants = self.transition_grants.read();
        let controllers = self.adapter_controllers.read();
        let candidates = state
            .leases
            .values()
            .filter(|lease| {
                lease.state == ResourceLeaseState::Active
                    && lease.control_mode == ResourceControlMode::Managed
                    && lease.adapter_id != request.adapter_id
                    && lease.priority < request.priority
            })
            .filter_map(|lease| {
                let descriptor = self.adapter_registry.descriptor(&lease.adapter_id)?;
                let operation = descriptor.automatic_preemption?;
                let restore_operation = restore_operation_for(operation)?;
                let authorized = grants
                    .get(&(request.adapter_id.clone(), lease.adapter_id.clone()))
                    .is_some_and(|operations| {
                        operations.contains(&operation) && operations.contains(&restore_operation)
                    });
                if !authorized || !controllers.contains_key(&lease.adapter_id) {
                    return None;
                }
                Some(AutomaticPreemptionCandidate {
                    adapter_id: lease.adapter_id.clone(),
                    profile_id: lease.profile_id.clone(),
                    operation,
                    restore_operation,
                    priority: lease.priority,
                    releasable_mib: lease.actual_mib.max(lease.reservation_mib),
                    releasable_ram_mib: lease.actual_ram_mib.max(lease.ram_reservation_mib),
                    releasable_cpu_threads: lease
                        .actual_cpu_threads
                        .max(lease.cpu_thread_reservation),
                    granted_at_ms: lease.granted_at_ms,
                })
            })
            .collect::<Vec<_>>();
        let mut by_adapter = BTreeMap::new();
        for candidate in candidates {
            by_adapter
                .entry(candidate.adapter_id.clone())
                .and_modify(|existing: &mut AutomaticPreemptionCandidate| {
                    existing.releasable_mib = existing
                        .releasable_mib
                        .saturating_add(candidate.releasable_mib);
                    existing.releasable_ram_mib = existing
                        .releasable_ram_mib
                        .saturating_add(candidate.releasable_ram_mib);
                    existing.releasable_cpu_threads = existing
                        .releasable_cpu_threads
                        .saturating_add(candidate.releasable_cpu_threads);
                    existing.priority = existing.priority.max(candidate.priority);
                    existing.granted_at_ms = existing.granted_at_ms.max(candidate.granted_at_ms);
                })
                .or_insert(candidate);
        }
        let mut candidates = by_adapter.into_values().collect::<Vec<_>>();
        candidates.sort_by_key(|candidate| {
            (
                candidate.priority,
                std::cmp::Reverse(
                    candidate
                        .releasable_mib
                        .saturating_add(candidate.releasable_ram_mib),
                ),
                std::cmp::Reverse(candidate.releasable_cpu_threads),
                std::cmp::Reverse(candidate.granted_at_ms),
                candidate.adapter_id.clone(),
            )
        });
        candidates
    }

    pub(super) fn diagnostics_from_state(
        &self,
        state: &CoordinatorState,
    ) -> ResourceCoordinationDiagnostics {
        let mut diagnostics = diagnostics_from_state(
            &self.policy,
            &self.adapter_registry,
            &self.controller_ids(),
            self.state_revision(),
            state,
        );
        diagnostics.admission_queue = self.admission_queue.diagnostics();
        diagnostics
    }

    pub(super) fn bump_revision(&self) {
        self.state_revision.fetch_add(1, Ordering::AcqRel);
    }
}

//! Lease bookkeeping: activation, release, and adapter state queries.

use super::{now_epoch_ms, prune_expired, ResourceCoordinator};

use oclive_kernel_types::{ResourceLeaseState, ResourcePriority};

impl ResourceCoordinator {
    pub fn activate(&self, lease_id: &str, actual_mib: Option<u64>) -> bool {
        self.activate_with_usage(lease_id, actual_mib, None, None)
    }

    /// Confirm runtime activation with optional measured GPU, RAM, and CPU
    /// usage. Missing measurements retain the admitted reservation.
    pub fn activate_with_usage(
        &self,
        lease_id: &str,
        actual_mib: Option<u64>,
        actual_ram_mib: Option<u64>,
        actual_cpu_threads: Option<u16>,
    ) -> bool {
        let now_ms = now_epoch_ms();
        let mut state = self.state.lock();
        let Some(lease) = state.leases.get_mut(lease_id) else {
            return false;
        };
        lease.state = ResourceLeaseState::Active;
        lease.actual_mib = actual_mib.unwrap_or(lease.reservation_mib);
        lease.actual_ram_mib = actual_ram_mib.unwrap_or(lease.ram_reservation_mib);
        lease.actual_cpu_threads = actual_cpu_threads.unwrap_or(lease.cpu_thread_reservation);
        lease.expires_at_ms =
            if lease.control_mode == oclive_kernel_types::ResourceControlMode::Managed {
                None
            } else {
                Some(now_ms.saturating_add(self.policy.active_lease_ttl_ms))
            };
        self.bump_revision();
        true
    }

    pub fn release(&self, lease_id: &str) -> bool {
        let removed = self.state.lock().leases.remove(lease_id).is_some();
        if removed {
            self.bump_revision();
        }
        removed
    }

    pub fn release_workload(&self, adapter_id: &str, workload_id: &str) -> usize {
        let mut state = self.state.lock();
        let before = state.leases.len();
        state
            .leases
            .retain(|_, lease| lease.adapter_id != adapter_id || lease.workload_id != workload_id);
        let released = before.saturating_sub(state.leases.len());
        if released > 0 {
            self.bump_revision();
        }
        released
    }

    pub fn release_adapter(&self, adapter_id: &str) -> usize {
        let mut state = self.state.lock();
        let before = state.leases.len();
        state
            .leases
            .retain(|_, lease| lease.adapter_id != adapter_id);
        let released = before.saturating_sub(state.leases.len());
        if released > 0 {
            self.bump_revision();
        }
        released
    }

    /// Attach a stable operational reason to every current lease for one adapter.
    ///
    /// Returns the number of leases that gained the reason. Repeated reasons are
    /// idempotent so retries do not grow diagnostics without bound.
    pub fn record_adapter_reason(&self, adapter_id: &str, reason_code: &str) -> usize {
        let reason_code = reason_code.trim();
        if adapter_id.trim().is_empty() || reason_code.is_empty() {
            return 0;
        }
        let mut state = self.state.lock();
        let mut updated = 0;
        for lease in state
            .leases
            .values_mut()
            .filter(|lease| lease.adapter_id == adapter_id)
        {
            if !lease
                .reason_codes
                .iter()
                .any(|existing| existing == reason_code)
            {
                lease.reason_codes.push(reason_code.to_string());
                updated += 1;
            }
        }
        if updated > 0 {
            self.bump_revision();
        }
        updated
    }

    #[must_use]
    pub fn adapter_has_reason(&self, adapter_id: &str, reason_code: &str) -> bool {
        let now_ms = now_epoch_ms();
        let mut state = self.state.lock();
        if prune_expired(&mut state.leases, now_ms) {
            self.bump_revision();
        }
        state.leases.values().any(|lease| {
            lease.adapter_id == adapter_id
                && lease
                    .reason_codes
                    .iter()
                    .any(|existing| existing == reason_code)
        })
    }

    #[must_use]
    pub fn has_active_priority(&self, minimum: ResourcePriority) -> bool {
        let now_ms = now_epoch_ms();
        let mut state = self.state.lock();
        if prune_expired(&mut state.leases, now_ms) {
            self.bump_revision();
        }
        state
            .leases
            .values()
            .any(|lease| lease.state == ResourceLeaseState::Active && lease.priority >= minimum)
    }

    #[must_use]
    pub fn has_active_adapter(&self, adapter_id: &str) -> bool {
        let now_ms = now_epoch_ms();
        let mut state = self.state.lock();
        if prune_expired(&mut state.leases, now_ms) {
            self.bump_revision();
        }
        state.leases.values().any(|lease| {
            lease.state == ResourceLeaseState::Active && lease.adapter_id == adapter_id
        })
    }

    #[must_use]
    /// Whether the adapter has a reserved or active lease after TTL pruning.
    pub fn has_adapter_lease(&self, adapter_id: &str) -> bool {
        let now_ms = now_epoch_ms();
        let mut state = self.state.lock();
        if prune_expired(&mut state.leases, now_ms) {
            self.bump_revision();
        }
        state
            .leases
            .values()
            .any(|lease| lease.adapter_id == adapter_id)
    }
}

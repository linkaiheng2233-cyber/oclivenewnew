use async_trait::async_trait;
use oclive_kernel_types::{
    ResourceAdapterOperation, ResourceAdapterRegistration, ResourceSnapshot, Result,
};
use std::sync::Arc;

/// Device-telemetry port consumed by the host-owned Resource Coordinator.
///
/// Implementations may use NVIDIA SMI, another vendor API, or a deterministic
/// test source. Failure is represented as an unavailable snapshot so callers
/// can apply the host's explicit unverified-admission policy.
#[async_trait]
pub trait ResourceSnapshotSource: Send + Sync {
    async fn snapshot(&self) -> ResourceSnapshot;
}

/// Confirmed outcome returned by a host-owned runtime controller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceAdapterControllerOutcome {
    pub already_in_state: bool,
    pub recovery_scheduled: bool,
}

/// Authoritative lifecycle port for one managed resource adapter.
///
/// Descriptors advertise what an adapter can do; registering this controller
/// proves which host process can actually perform those operations. The
/// Resource Coordinator serializes calls and validates descriptor/profile
/// claims before invoking this port.
#[async_trait]
pub trait ResourceAdapterController: Send + Sync {
    fn adapter_id(&self) -> &str;

    /// Apply one validated lifecycle operation.
    ///
    /// # Errors
    ///
    /// Returns an adapter-specific runtime error when the operation cannot be
    /// confirmed. Callers must not infer resource release from a failed call.
    async fn transition(
        &self,
        operation: ResourceAdapterOperation,
        profile_id: Option<&str>,
        reason: Option<&str>,
    ) -> Result<ResourceAdapterControllerOutcome>;
}

/// Owner-scoped host entry point for third-party resource adapters.
///
/// The registrar accepts control-plane facts and an optional single-writer
/// controller. It does not load plugins, execute untrusted manifests, grant
/// cross-adapter transitions, or carry business data.
pub trait ResourceAdapterRegistrar: Send + Sync {
    /// Register one namespaced descriptor owned by `registration.source_id`.
    ///
    /// # Errors
    ///
    /// Returns a validation or conflict error when the source cannot own the
    /// adapter namespace, the descriptor is invalid, or an incompatible
    /// registration already exists.
    fn register_adapter(&self, registration: ResourceAdapterRegistration) -> Result<()>;

    /// Bind the owner-provided controller for a previously registered managed
    /// descriptor.
    ///
    /// # Errors
    ///
    /// Returns a validation or ownership error when the source does not own
    /// the controller's registered adapter descriptor.
    fn register_controller(
        &self,
        source_id: &str,
        controller: Arc<dyn ResourceAdapterController>,
    ) -> Result<()>;
}

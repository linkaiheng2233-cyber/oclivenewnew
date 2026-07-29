use async_trait::async_trait;
use oclive_kernel_types::ResourceSnapshot;

/// Device-telemetry port consumed by the host-owned Resource Coordinator.
///
/// Implementations may use NVIDIA SMI, another vendor API, or a deterministic
/// test source. Failure is represented as an unavailable snapshot so callers
/// can apply the host's explicit unverified-admission policy.
#[async_trait]
pub trait ResourceSnapshotSource: Send + Sync {
    async fn snapshot(&self) -> ResourceSnapshot;
}

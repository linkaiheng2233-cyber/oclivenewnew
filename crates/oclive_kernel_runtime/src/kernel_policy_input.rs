//! Shared policy input builder for CLI and desktop hosts.

use crate::kernel_discovery::KernelCandidate;
use crate::kernel_distro_profile::{
    evaluate_profile_compat, resolve_caller_requirements,
};
use crate::kernel_manifest::KernelBinaryManifest;
use crate::kernel_runtime_health::profile_file_sha256_hex;
use crate::kernel_strategy::{resolve_kernel_action, KernelActionPlan, ResolveKernelActionInput};
use oclive_kernel_types::{
    ActiveProfileSummary, DistroProfileRequirements, KernelHealthJson, ProfileCompat,
};
use std::path::Path;

/// Inputs collected from health probe + host context.
pub struct PolicyContext {
    pub health_ok: bool,
    pub running_manifest: Option<KernelBinaryManifest>,
    pub running_distro_id: Option<String>,
    pub running_profile: Option<ActiveProfileSummary>,
    pub running_profile_hash: Option<String>,
}

impl PolicyContext {
    #[must_use]
    pub fn from_health(health: Option<&KernelHealthJson>) -> Self {
        let health_ok = health.is_some_and(|h| h.ok);
        Self {
            health_ok,
            running_manifest: None,
            running_distro_id: health.and_then(|h| h.distro_id.clone()),
            running_profile: health.and_then(|h| h.active_profile_summary.clone()),
            running_profile_hash: health.and_then(|h| h.distro_profile_hash.clone()),
        }
    }

    #[must_use]
    pub fn with_manifest(mut self, manifest: Option<KernelBinaryManifest>) -> Self {
        self.running_manifest = manifest;
        self
    }
}

/// Resolved plan + diagnostics for `EnsureReport` / logging.
pub struct PolicyResolution {
    pub plan: KernelActionPlan,
    pub profile_compat: ProfileCompat,
    pub caller_requirements: DistroProfileRequirements,
    pub running_profile_summary: Option<ActiveProfileSummary>,
}

/// Build policy input and resolve action (SSOT for CLI + desktop).
#[must_use]
pub fn build_resolve_plan(
    ctx: &PolicyContext,
    candidates: &[KernelCandidate],
    distro_id: &str,
    profile_path: Option<&Path>,
    kernel_pinned: bool,
    allow_replace_running: bool,
    promote_shared: bool,
) -> PolicyResolution {
    let caller_requirements = resolve_caller_requirements(distro_id, profile_path);
    let caller_profile_hash = profile_path.and_then(profile_file_sha256_hex);

    let profile_compat = evaluate_profile_compat(
        &caller_requirements,
        ctx.running_profile.as_ref(),
        ctx.running_distro_id.as_deref(),
        ctx.running_profile_hash.as_deref(),
        caller_profile_hash.as_deref(),
    );

    let input = ResolveKernelActionInput {
        running: ctx.running_manifest.as_ref(),
        running_health_ok: ctx.health_ok,
        candidates,
        kernel_pinned,
        caller_distro_id: Some(distro_id),
        caller_requirements: Some(&caller_requirements),
        running_profile: ctx.running_profile.as_ref(),
        running_distro_id: ctx.running_distro_id.as_deref(),
        running_profile_hash: ctx.running_profile_hash.as_deref(),
        caller_profile_hash: caller_profile_hash.as_deref(),
        allow_replace_running,
        promote_shared,
    };

    PolicyResolution {
        plan: resolve_kernel_action(&input),
        profile_compat,
        caller_requirements,
        running_profile_summary: ctx.running_profile.clone(),
    }
}

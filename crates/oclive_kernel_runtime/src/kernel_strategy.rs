//! Kernel scheduling policy — pure decision logic (SSOT for all distros).
//!
//! ## Decision priority (healthy kernel)
//!
//! 1. **User pin** → attach (`KernelPinned` / `KernelPinnedProfileMismatch`).
//! 2. **Profile compatibility** (summary / hash / satisfies caller) → attach even when a fuller binary exists.
//! 3. **Profile mismatch** → `ReplaceAndAttach` with `ReplaceReason::ProfileMismatch`.
//! 4. **Profile unknown** + weaker binary → `ReplaceReason::BinaryUpgrade`.
//!
//! Offline: spawn best candidate; bundled-only → fallback.

use crate::kernel_discovery::{KernelCandidate, KernelTier, PROMOTE_SCORE_THRESHOLD};
use crate::kernel_distro_profile::evaluate_profile_compat;
use crate::kernel_manifest::KernelBinaryManifest;
use oclive_kernel_types::{
    ActiveProfileSummary, AttachReason, DistroProfileRequirements, ProfileCompat, ReplaceReason,
};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KernelActionKind {
    Attach,
    ReplaceAndAttach,
    SpawnBest,
    FallbackBundled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KernelActionCandidate {
    pub binary: String,
    pub tier: KernelTier,
    pub score: u8,
    pub promote_to_shared: bool,
    pub degraded: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub degrade_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KernelActionPlan {
    pub action: KernelActionKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidate: Option<KernelActionCandidate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attach_reason: Option<AttachReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replace_reason: Option<ReplaceReason>,
    pub degraded: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub degrade_reason: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ResolveKernelActionInput<'a> {
    pub running: Option<&'a KernelBinaryManifest>,
    pub running_health_ok: bool,
    pub candidates: &'a [KernelCandidate],
    pub kernel_pinned: bool,
    pub caller_distro_id: Option<&'a str>,
    pub caller_requirements: Option<&'a DistroProfileRequirements>,
    pub running_profile: Option<&'a ActiveProfileSummary>,
    pub running_distro_id: Option<&'a str>,
    pub running_profile_hash: Option<&'a str>,
    pub caller_profile_hash: Option<&'a str>,
    pub allow_replace_running: bool,
    pub promote_shared: bool,
}

#[must_use]
pub fn manifest_for_candidate(candidate: &KernelCandidate) -> KernelBinaryManifest {
    if let Some(m) = KernelBinaryManifest::read_sidecar(&candidate.binary) {
        return m;
    }
    let build_profile = match candidate.tier {
        KernelTier::Bundled => "bundled",
        _ => "full",
    };
    KernelBinaryManifest::synthetic(build_profile, "0.0.0")
}

#[must_use]
pub fn pick_best_by_capability(candidates: &[KernelCandidate]) -> Option<usize> {
    candidates
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| {
            let ma = manifest_for_candidate(a);
            let mb = manifest_for_candidate(b);
            ma.cmp_for_capability(&mb)
                .then_with(|| a.score.cmp(&b.score))
        })
        .map(|(i, _)| i)
}

fn profile_compat(input: &ResolveKernelActionInput<'_>) -> ProfileCompat {
    let Some(caller) = input.caller_requirements else {
        return ProfileCompat::Unknown;
    };
    evaluate_profile_compat(
        caller,
        input.running_profile,
        input.running_distro_id,
        input.running_profile_hash,
        input.caller_profile_hash,
    )
}

fn running_weaker_than_best(input: &ResolveKernelActionInput<'_>) -> bool {
    let Some(running) = input.running else {
        return false;
    };
    let Some(idx) = pick_best_by_capability(input.candidates) else {
        return false;
    };
    let best = manifest_for_candidate(&input.candidates[idx]);
    matches!(running.cmp_for_capability(&best), Ordering::Less)
}

fn try_replace_plan(
    input: &ResolveKernelActionInput<'_>,
    reason: ReplaceReason,
) -> Option<KernelActionPlan> {
    if !input.allow_replace_running {
        return None;
    }
    let idx = pick_best_by_capability(input.candidates)?;
    let best = &input.candidates[idx];
    Some(KernelActionPlan {
        action: KernelActionKind::ReplaceAndAttach,
        candidate: Some(action_candidate(
            best,
            input.candidates,
            input.promote_shared,
        )),
        attach_reason: None,
        replace_reason: Some(reason),
        degraded: false,
        degrade_reason: None,
    })
}

fn resolve_healthy(input: &ResolveKernelActionInput<'_>) -> KernelActionPlan {
    let compat = profile_compat(input);

    if input.kernel_pinned {
        let reason = match compat {
            ProfileCompat::Incompatible => AttachReason::KernelPinnedProfileMismatch,
            _ => AttachReason::KernelPinned,
        };
        return KernelActionPlan {
            action: KernelActionKind::Attach,
            candidate: None,
            attach_reason: Some(reason),
            replace_reason: None,
            degraded: false,
            degrade_reason: None,
        };
    }

    match compat {
        ProfileCompat::Incompatible => {
            if let Some(plan) = try_replace_plan(input, ReplaceReason::ProfileMismatch) {
                return plan;
            }
            KernelActionPlan {
                action: KernelActionKind::Attach,
                candidate: None,
                attach_reason: Some(AttachReason::ProfileMismatchNoReplace),
                replace_reason: None,
                degraded: false,
                degrade_reason: None,
            }
        }
        ProfileCompat::Compatible => KernelActionPlan {
            action: KernelActionKind::Attach,
            candidate: None,
            attach_reason: Some(AttachReason::ProfileCompatible),
            replace_reason: None,
            degraded: false,
            degrade_reason: None,
        },
        ProfileCompat::Unknown => {
            if running_weaker_than_best(input) {
                if let Some(plan) = try_replace_plan(input, ReplaceReason::BinaryUpgrade) {
                    return plan;
                }
            }
            KernelActionPlan {
                action: KernelActionKind::Attach,
                candidate: None,
                attach_reason: Some(AttachReason::RunningKernelOk),
                replace_reason: None,
                degraded: false,
                degrade_reason: None,
            }
        }
    }
}

/// Cross-host kernel lifecycle policy: merges health, candidates, and caller requirements into attach/spawn/replace.
#[must_use]
pub fn resolve_kernel_action(input: &ResolveKernelActionInput<'_>) -> KernelActionPlan {
    if input.running_health_ok {
        return resolve_healthy(input);
    }

    if input.candidates.is_empty() {
        return KernelActionPlan {
            action: KernelActionKind::FallbackBundled,
            candidate: None,
            attach_reason: None,
            replace_reason: None,
            degraded: true,
            degrade_reason: Some("no_kernel_candidates".into()),
        };
    }

    if input.kernel_pinned {
        if let Some(c) = pinned_candidate(input.candidates) {
            let candidate = action_candidate(c, input.candidates, input.promote_shared);
            return KernelActionPlan {
                action: KernelActionKind::SpawnBest,
                candidate: Some(candidate),
                attach_reason: Some(AttachReason::KernelPinned),
                replace_reason: None,
                degraded: false,
                degrade_reason: None,
            };
        }
    }

    let non_bundled: Vec<usize> = input
        .candidates
        .iter()
        .enumerate()
        .filter(|(_, c)| c.tier != KernelTier::Bundled)
        .map(|(i, _)| i)
        .collect();

    let pick_idx = if non_bundled.is_empty() {
        pick_best_by_capability(input.candidates)
    } else {
        pick_best_by_capability(
            &non_bundled
                .iter()
                .map(|&i| input.candidates[i].clone())
                .collect::<Vec<_>>(),
        )
        .map(|j| non_bundled[j])
    };

    let Some(idx) = pick_idx else {
        return KernelActionPlan {
            action: KernelActionKind::FallbackBundled,
            candidate: None,
            attach_reason: None,
            replace_reason: None,
            degraded: true,
            degrade_reason: Some("no_spawn_candidate".into()),
        };
    };

    let best = &input.candidates[idx];
    let only_bundled = input
        .candidates
        .iter()
        .all(|c| c.tier == KernelTier::Bundled);
    let candidate = action_candidate(best, input.candidates, input.promote_shared);

    if only_bundled || best.tier == KernelTier::Bundled {
        KernelActionPlan {
            action: KernelActionKind::FallbackBundled,
            candidate: Some(KernelActionCandidate {
                degraded: true,
                degrade_reason: Some(
                    "no_shared_or_dev_kernel; using bundled fallback".into(),
                ),
                ..candidate
            }),
            attach_reason: None,
            replace_reason: None,
            degraded: true,
            degrade_reason: candidate.degrade_reason.clone(),
        }
    } else {
        KernelActionPlan {
            action: KernelActionKind::SpawnBest,
            candidate: Some(candidate),
            attach_reason: None,
            replace_reason: None,
            degraded: false,
            degrade_reason: None,
        }
    }
}

fn pinned_candidate(candidates: &[KernelCandidate]) -> Option<&KernelCandidate> {
    candidates
        .iter()
        .find(|c| matches!(c.tier, KernelTier::Env | KernelTier::Settings))
        .or_else(|| candidates.first())
}

fn action_candidate(
    candidate: &KernelCandidate,
    all: &[KernelCandidate],
    promote_shared: bool,
) -> KernelActionCandidate {
    let promote_to_shared = promote_shared
        && candidate.score >= PROMOTE_SCORE_THRESHOLD
        && !matches!(candidate.tier, KernelTier::Shared | KernelTier::Bundled);
    let only_bundled = all.iter().all(|c| c.tier == KernelTier::Bundled);
    KernelActionCandidate {
        binary: candidate.binary.display().to_string(),
        tier: candidate.tier,
        score: candidate.score,
        promote_to_shared,
        degraded: only_bundled && candidate.tier == KernelTier::Bundled,
        degrade_reason: if only_bundled && candidate.tier == KernelTier::Bundled {
            Some("no_shared_or_dev_kernel; using bundled fallback".into())
        } else {
            None
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel_distro_profile::default_requirements_for_distro_id;
    use std::path::PathBuf;

    fn full_manifest() -> KernelBinaryManifest {
        KernelBinaryManifest::synthetic("full", "0.3.0")
    }

    fn bundled_manifest() -> KernelBinaryManifest {
        KernelBinaryManifest::synthetic("bundled", "0.3.0")
    }

    fn candidate(tier: KernelTier, score: u8, name: &str) -> KernelCandidate {
        KernelCandidate {
            binary: PathBuf::from(name),
            tier,
            score,
            extra_args: vec![],
        }
    }

    fn active_for(distro_id: &str) -> ActiveProfileSummary {
        crate::kernel_distro_profile::active_summary_from_requirements(
            &default_requirements_for_distro_id(distro_id),
        )
    }

    #[test]
    fn attach_when_profile_compatible_despite_older_binary() {
        let cands = vec![candidate(KernelTier::DevHeadless, 90, "/dev/kernel")];
        let desktop = default_requirements_for_distro_id("desktop");
        let input = ResolveKernelActionInput {
            running: Some(&full_manifest()),
            running_health_ok: true,
            candidates: &cands,
            kernel_pinned: false,
            caller_distro_id: Some("desktop"),
            caller_requirements: Some(&desktop),
            running_profile: Some(&active_for("desktop")),
            running_distro_id: Some("desktop"),
            running_profile_hash: None,
            caller_profile_hash: None,
            allow_replace_running: true,
            promote_shared: true,
        };
        let plan = resolve_kernel_action(&input);
        assert_eq!(plan.action, KernelActionKind::Attach);
        assert_eq!(plan.attach_reason, Some(AttachReason::ProfileCompatible));
    }

    #[test]
    fn replace_when_vscode_calls_desktop_kernel() {
        let cands = vec![candidate(KernelTier::Shared, 88, "/shared/kernel")];
        let vscode = default_requirements_for_distro_id("vscode");
        let input = ResolveKernelActionInput {
            running: Some(&full_manifest()),
            running_health_ok: true,
            candidates: &cands,
            kernel_pinned: false,
            caller_distro_id: Some("vscode"),
            caller_requirements: Some(&vscode),
            running_profile: Some(&active_for("desktop")),
            running_distro_id: Some("desktop"),
            running_profile_hash: None,
            caller_profile_hash: None,
            allow_replace_running: true,
            promote_shared: true,
        };
        let plan = resolve_kernel_action(&input);
        assert_eq!(plan.action, KernelActionKind::ReplaceAndAttach);
        assert_eq!(plan.replace_reason, Some(ReplaceReason::ProfileMismatch));
    }

    #[test]
    fn same_distro_id_hash_mismatch_unknown_triggers_binary_upgrade() {
        let cands = vec![candidate(KernelTier::DevHeadless, 90, "/dev/kernel")];
        let desktop = default_requirements_for_distro_id("desktop");
        let input = ResolveKernelActionInput {
            running: Some(&bundled_manifest()),
            running_health_ok: true,
            candidates: &cands,
            kernel_pinned: false,
            caller_distro_id: Some("desktop"),
            caller_requirements: Some(&desktop),
            running_profile: None,
            running_distro_id: Some("desktop"),
            running_profile_hash: Some("aaa"),
            caller_profile_hash: Some("bbb"),
            allow_replace_running: true,
            promote_shared: true,
        };
        let plan = resolve_kernel_action(&input);
        assert_eq!(plan.action, KernelActionKind::ReplaceAndAttach);
        assert_eq!(plan.replace_reason, Some(ReplaceReason::BinaryUpgrade));
    }

    #[test]
    fn pinned_profile_mismatch_still_attaches() {
        let cands = vec![candidate(KernelTier::DevHeadless, 90, "/dev/kernel")];
        let vscode = default_requirements_for_distro_id("vscode");
        let input = ResolveKernelActionInput {
            running: Some(&full_manifest()),
            running_health_ok: true,
            candidates: &cands,
            kernel_pinned: true,
            caller_distro_id: Some("vscode"),
            caller_requirements: Some(&vscode),
            running_profile: Some(&active_for("desktop")),
            running_distro_id: Some("desktop"),
            running_profile_hash: None,
            caller_profile_hash: None,
            allow_replace_running: true,
            promote_shared: true,
        };
        let plan = resolve_kernel_action(&input);
        assert_eq!(plan.action, KernelActionKind::Attach);
        assert_eq!(
            plan.attach_reason,
            Some(AttachReason::KernelPinnedProfileMismatch)
        );
    }

    #[test]
    fn spawn_best_when_offline() {
        let cands = vec![
            candidate(KernelTier::Shared, 88, "/shared/kernel"),
            candidate(KernelTier::Bundled, 50, "/bundled/kernel"),
        ];
        let input = ResolveKernelActionInput {
            running: None,
            running_health_ok: false,
            candidates: &cands,
            kernel_pinned: false,
            caller_distro_id: Some("vscode"),
            caller_requirements: None,
            running_profile: None,
            running_distro_id: None,
            running_profile_hash: None,
            caller_profile_hash: None,
            allow_replace_running: true,
            promote_shared: true,
        };
        let plan = resolve_kernel_action(&input);
        assert_eq!(plan.action, KernelActionKind::SpawnBest);
    }
}

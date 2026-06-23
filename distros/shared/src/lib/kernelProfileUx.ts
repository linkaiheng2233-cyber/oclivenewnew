import type { KernelConnectionStatus } from '@oclive/shared/api/kernel'

/** Maps backend profile hint keys to i18n label keys (status bar). */
const STATUS_LABEL_BY_HINT: Record<string, string> = {
  profile_compatible: 'kernel.status.connectedLocal',
  profile_mismatch_no_replace: 'kernel.status.connectedProfileMismatch',
  kernel_pinned_profile_mismatch: 'kernel.status.connectedPinnedMismatch',
  replaced_for_profile: 'kernel.status.connectedReplaced',
  degraded: 'kernel.status.connectedDegraded',
  legacy_fallback: 'kernel.status.connectedDegraded',
}

const SETTINGS_DETAIL_BY_HINT: Record<string, string> = {
  profile_compatible: 'kernel.profile.detailCompatible',
  profile_mismatch_no_replace: 'kernel.profile.detailMismatch',
  kernel_pinned_profile_mismatch: 'kernel.profile.detailPinnedMismatch',
  replaced_for_profile: 'kernel.profile.detailReplaced',
  degraded: 'kernel.profile.detailDegraded',
  legacy_fallback: 'kernel.profile.detailDegraded',
}

export function kernelStatusLabelKey(status: KernelConnectionStatus | null | undefined): string {
  if (!status?.healthy) {
    return 'kernel.status.offlineTapReconnect'
  }
  const hint = status.profileHintKey?.trim()
  if (hint && STATUS_LABEL_BY_HINT[hint]) {
    return STATUS_LABEL_BY_HINT[hint]
  }
  if (status.degraded) {
    return 'kernel.status.connectedDegraded'
  }
  if (status.mode === 'spawned') {
    return 'kernel.status.spawned'
  }
  if (status.mode === 'attached') {
    return 'kernel.status.connectedLocal'
  }
  return 'kernel.status.connectedLocal'
}

export function kernelProfileDetailKey(
  status: KernelConnectionStatus | null | undefined,
): string | null {
  if (!status?.healthy) {
    return null
  }
  const hint = status.profileHintKey?.trim()
  if (hint && SETTINGS_DETAIL_BY_HINT[hint]) {
    return SETTINGS_DETAIL_BY_HINT[hint]
  }
  if (status.degraded) {
    return 'kernel.profile.detailDegraded'
  }
  if (status.mode === 'attached' || status.mode === 'spawned') {
    return 'kernel.profile.detailCompatible'
  }
  return null
}

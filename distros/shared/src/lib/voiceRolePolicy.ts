export const VOICE_ROLE_TTS_ENABLED_KEY = 'role_tts_enabled'

export type VoiceRoleTtsEnabledMap = Record<string, true>

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
}

function normalizedRoleId(value: unknown): string {
  return typeof value === 'string' ? value.trim() : ''
}

/** Keep only explicit enabled entries; missing and false entries both mean disabled. */
export function normalizeVoiceRoleTtsEnabled(
  value: unknown,
): VoiceRoleTtsEnabledMap {
  if (!isRecord(value))
    return {}
  const normalized: VoiceRoleTtsEnabledMap = {}
  for (const [rawRoleId, enabled] of Object.entries(value)) {
    const roleId = normalizedRoleId(rawRoleId)
    if (roleId && enabled === true)
      normalized[roleId] = true
  }
  return normalized
}

/** Whether this config uses the new explicit per-role policy rather than legacy pack detection. */
export function hasExplicitVoiceRoleTtsPolicy(
  config: Record<string, unknown>,
): boolean {
  return Object.hasOwn(config, VOICE_ROLE_TTS_ENABLED_KEY)
    && isRecord(config[VOICE_ROLE_TTS_ENABLED_KEY])
}

/**
 * Resolve an explicit role decision.
 * Returns `null` for legacy configs so callers can fall back to voice-profile detection.
 */
export function explicitVoiceRoleTtsDecision(
  config: Record<string, unknown>,
  roleId: string,
): boolean | null {
  if (!hasExplicitVoiceRoleTtsPolicy(config))
    return null
  const rid = normalizedRoleId(roleId)
  if (!rid)
    return false
  return normalizeVoiceRoleTtsEnabled(
    config[VOICE_ROLE_TTS_ENABLED_KEY],
  )[rid] === true
}

/** Startup warm is useful only when a new-policy config enables at least one role. */
export function hasAnyExplicitVoiceRoleEnabled(
  config: Record<string, unknown>,
): boolean | null {
  if (!hasExplicitVoiceRoleTtsPolicy(config))
    return null
  return Object.keys(
    normalizeVoiceRoleTtsEnabled(config[VOICE_ROLE_TTS_ENABLED_KEY]),
  ).length > 0
}

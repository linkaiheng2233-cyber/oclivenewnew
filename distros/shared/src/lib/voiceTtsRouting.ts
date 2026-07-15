export type VoiceTtsProfileRouting = {
  engine?: string
  synth_provider?: string
  sidecar_endpoint?: string
}

export type VoiceTtsRuntimeRouting = {
  tts_profile: string
  tts_engine: string
  synth_provider: string
  local_synth_endpoint: string
}

/**
 * Resolve one speak job without mutating the user's global voice settings.
 * A role directive may override the profile; roles without an override inherit
 * the globally selected TTS infrastructure.
 */
export function resolveVoiceTtsRouting(
  globalRouting: VoiceTtsRuntimeRouting,
  roleProfile: string | undefined,
  profiles: ReadonlyMap<string, VoiceTtsProfileRouting>,
): VoiceTtsRuntimeRouting {
  const requestedRoleProfile = roleProfile?.trim()
  const roleRouting = requestedRoleProfile
    ? profiles.get(requestedRoleProfile)
    : undefined
  const profileId = roleRouting
    ? requestedRoleProfile!
    : globalRouting.tts_profile
  const profile = roleRouting || profiles.get(profileId)
  return {
    tts_profile: profileId,
    tts_engine: profile?.engine || globalRouting.tts_engine,
    synth_provider: profile?.synth_provider || globalRouting.synth_provider,
    local_synth_endpoint:
      profile?.sidecar_endpoint || globalRouting.local_synth_endpoint,
  }
}

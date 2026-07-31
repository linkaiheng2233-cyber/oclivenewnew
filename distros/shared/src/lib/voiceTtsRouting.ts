export interface VoiceTtsProfileRouting {
  engine?: string
  synth_provider?: string
  sidecar_endpoint?: string
}

export interface VoiceTtsRuntimeRouting {
  tts_profile: string
  tts_engine: string
  synth_provider: string
  local_synth_endpoint: string
}

/**
 * Resolve one speak job without mutating the user's global voice settings.
 * A role directive may override the profile; roles without an override inherit
 * the globally selected TTS infrastructure (including settings-page provider
 * / endpoint overrides that may differ from catalog defaults).
 *
 * `build_directive` fills `synth_profile` with the global profile for packs
 * that omit one — callers may pass that value; matching the global id must
 * not re-apply catalog defaults over the user's runtime config.
 */
export function resolveVoiceTtsRouting(
  globalRouting: VoiceTtsRuntimeRouting,
  roleProfile: string | undefined,
  profiles: ReadonlyMap<string, VoiceTtsProfileRouting>,
): VoiceTtsRuntimeRouting {
  const requestedRoleProfile = roleProfile?.trim() || undefined
  if (!requestedRoleProfile || requestedRoleProfile === globalRouting.tts_profile) {
    return { ...globalRouting }
  }
  const roleRouting = profiles.get(requestedRoleProfile)
  if (!roleRouting) {
    return { ...globalRouting }
  }
  return {
    tts_profile: requestedRoleProfile,
    tts_engine: roleRouting.engine || globalRouting.tts_engine,
    synth_provider: roleRouting.synth_provider || globalRouting.synth_provider,
    local_synth_endpoint:
      roleRouting.sidecar_endpoint || globalRouting.local_synth_endpoint,
  }
}

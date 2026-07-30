import { directoryPluginInvoke, getPluginSettingsUi } from '@oclive/shared/api'
import { VOICE_ASR_PLUGIN_ID } from '@oclive/shared/lib/voiceAsrEvents'
import { hasAnyExplicitVoiceRoleEnabled } from '@oclive/shared/lib/voiceRolePolicy'

const DEFAULT_TTS_PROFILE = 'bundled-cosyvoice2-zh'
const warmPromises = new Map<string, Promise<void>>()
const cachedSidecarEndpoints = new Map<string, string>()

export interface VoiceWarmDirective {
  emo_text?: string
  ref_audio?: string
  ref_text?: string
  speed?: number
}

export interface VoiceWarmOptions {
  profile?: string
  directive?: VoiceWarmDirective
}

interface VoiceWarmResult {
  ok?: boolean
  warmed?: boolean
  already_warmed?: boolean
  sidecar_endpoint?: string
  sidecar_ready?: boolean
}

function normalizeWarmDirective(
  directive?: VoiceWarmDirective,
): VoiceWarmDirective | undefined {
  if (!directive)
    return undefined
  const normalized: VoiceWarmDirective = {
    emo_text: directive.emo_text?.trim() || undefined,
    ref_audio: directive.ref_audio?.trim() || undefined,
    ref_text: directive.ref_text?.trim() || undefined,
    speed: typeof directive.speed === 'number' ? directive.speed : undefined,
  }
  return normalized.emo_text || normalized.ref_audio ? normalized : undefined
}

function warmKey(profile: string, directive?: VoiceWarmDirective): string {
  return `${profile}|${JSON.stringify(normalizeWarmDirective(directive) || {})}`
}

function rememberSidecarEndpoint(
  profile: string,
  result: unknown,
  hostAdmittedWarm = false,
): void {
  const candidate = result as VoiceWarmResult | null
  // The bundled sidecar may be reachable while its GPU warm request was denied.
  // Expose it to direct browser streaming only after `voice.warm` passed through
  // the host Resource Coordinator. External/custom profiles remain user-owned.
  if (profile === DEFAULT_TTS_PROFILE && !hostAdmittedWarm)
    return
  if (candidate?.ok !== true && candidate?.sidecar_ready !== true)
    return
  const ep = candidate.sidecar_endpoint?.trim()
  if (ep)
    cachedSidecarEndpoints.set(profile, ep)
}

function isSidecarAlreadyWarmed(result: unknown): boolean {
  const r = result as VoiceWarmResult & { skipped?: boolean } | null
  if (r?.skipped)
    return true
  if (!r?.ok)
    return false
  return r.warmed === true || r.already_warmed === true
}

/** Sidecar URL from the last successful plugin warm/probe (authoritative for stream fetch). */
export function getVoiceSidecarEndpoint(profile = DEFAULT_TTS_PROFILE): string | null {
  return cachedSidecarEndpoints.get(profile) || null
}

export function resetVoiceSidecarEndpoint(profile?: string): void {
  if (profile)
    cachedSidecarEndpoints.delete(profile)
  else
    cachedSidecarEndpoints.clear()
}

/** Clear in-flight warm (e.g. after voice settings change). */
export function resetVoiceExpansionWarmSchedule(): void {
  warmPromises.clear()
  resetVoiceSidecarEndpoint()
}

async function probeTtsSidecar(
  profile: string,
): Promise<VoiceWarmResult | null> {
  try {
    return (await directoryPluginInvoke(
      VOICE_ASR_PLUGIN_ID,
      'voice.probe_tts',
      { profile },
    )) as VoiceWarmResult
  }
  catch {
    return null
  }
}

function startBackgroundWarm(
  profile: string,
  directive?: VoiceWarmDirective,
  knownProbe?: VoiceWarmResult | null,
): void {
  const key = warmKey(profile, directive)
  if (warmPromises.has(key))
    return
  const promise = (async () => {
    const probe = knownProbe === undefined
      ? await probeTtsSidecar(profile)
      : knownProbe
    if (probe)
      rememberSidecarEndpoint(profile, probe)
    const roleDirective = normalizeWarmDirective(directive)
    if (
      profile !== DEFAULT_TTS_PROFILE
      && isSidecarAlreadyWarmed(probe)
      && !roleDirective
    ) {
      return
    }
    const result = await directoryPluginInvoke(VOICE_ASR_PLUGIN_ID, 'voice.warm', {
      profile,
      ...(roleDirective ? { directive: roleDirective } : {}),
    })
    rememberSidecarEndpoint(profile, result, true)
  })().catch(() => {}).finally(() => {
    if (warmPromises.get(key) === promise)
      warmPromises.delete(key)
  })
  warmPromises.set(key, promise)
}

/**
 * If voice expansion is enabled in saved settings, warm CosyVoice sidecar once (deduped).
 * Does not block callers for the full warm duration — skips when probe reports already warmed.
 */
export async function scheduleVoiceExpansionWarm(
  isPluginDisabled: (id: string) => boolean = () => false,
  options: VoiceWarmOptions = {},
): Promise<void> {
  if (isPluginDisabled(VOICE_ASR_PLUGIN_ID))
    return
  try {
    const ui = await getPluginSettingsUi(VOICE_ASR_PLUGIN_ID)
    const cfg = ui.config ?? {}
    if (cfg.tts_expansion_enabled !== true)
      return
    if (hasAnyExplicitVoiceRoleEnabled(cfg) === false)
      return
    const profile = options.profile?.trim()
      || (typeof cfg.tts_profile === 'string' && cfg.tts_profile.trim()
        ? cfg.tts_profile.trim()
        : DEFAULT_TTS_PROFILE)
    const probe = await probeTtsSidecar(profile)
    if (probe)
      rememberSidecarEndpoint(profile, probe)
    const roleDirective = normalizeWarmDirective(options.directive)
    if (
      profile !== DEFAULT_TTS_PROFILE
      && isSidecarAlreadyWarmed(probe)
      && !roleDirective
    ) {
      return
    }
    startBackgroundWarm(profile, roleDirective, probe)
    if (cachedSidecarEndpoints.has(profile))
      return
  }
  catch {}
}

/**
 * Resolve sidecar URL for stream fetch without blocking on a long warm/prime cycle.
 * Kicks off background warm and uses probe_tts for the authoritative endpoint.
 */
export async function resolveVoiceSidecarEndpoint(
  profile: string,
  _fallbackEndpoint: string,
  isPluginDisabled: (id: string) => boolean = () => false,
): Promise<string | null> {
  const cachedEndpoint = cachedSidecarEndpoints.get(profile)
  if (cachedEndpoint)
    return cachedEndpoint

  if (!isPluginDisabled(VOICE_ASR_PLUGIN_ID)) {
    const probe = await probeTtsSidecar(profile)
    if (probe)
      rememberSidecarEndpoint(profile, probe)
    if (!isSidecarAlreadyWarmed(probe)) {
      try {
        const ui = await getPluginSettingsUi(VOICE_ASR_PLUGIN_ID)
        if (
          ui.config?.tts_expansion_enabled === true
          && hasAnyExplicitVoiceRoleEnabled(ui.config) !== false
        ) {
          startBackgroundWarm(profile, undefined, probe)
        }
      }
      catch {
        /* ignore */
      }
    }
    const resolvedEndpoint = cachedSidecarEndpoints.get(profile)
    if (resolvedEndpoint)
      return resolvedEndpoint
  }

  return null
}

/** Fire-and-forget startup warm after plugins are ready. */
export function startVoiceExpansionWarmOnStartup(
  isPluginDisabled: (id: string) => boolean = () => false,
): void {
  void scheduleVoiceExpansionWarm(isPluginDisabled)
}

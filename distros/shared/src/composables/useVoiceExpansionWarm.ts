import { directoryPluginInvoke, getPluginSettingsUi } from '@oclive/shared/api'
import { VOICE_ASR_PLUGIN_ID } from '@oclive/shared/lib/voiceAsrEvents'

const DEFAULT_TTS_PROFILE = 'bundled-cosyvoice2-zh'
const DEFAULT_SIDECAR = 'http://127.0.0.1:50000'

let warmPromise: Promise<void> | null = null
let cachedSidecarEndpoint: string | null = null

type VoiceWarmResult = {
  ok?: boolean
  warmed?: boolean
  already_warmed?: boolean
  sidecar_endpoint?: string
}

function rememberSidecarEndpoint(result: unknown): void {
  const ep = (result as VoiceWarmResult | null)?.sidecar_endpoint?.trim()
  if (ep)
    cachedSidecarEndpoint = ep
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
export function getVoiceSidecarEndpoint(): string | null {
  return cachedSidecarEndpoint
}

export function resetVoiceSidecarEndpoint(): void {
  cachedSidecarEndpoint = null
}

/** Clear in-flight warm (e.g. after voice settings change). */
export function resetVoiceExpansionWarmSchedule(): void {
  warmPromise = null
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
): void {
  if (warmPromise)
    return
  warmPromise = (async () => {
    const probe = await probeTtsSidecar(profile)
    if (probe)
      rememberSidecarEndpoint(probe)
    if (isSidecarAlreadyWarmed(probe))
      return
    const result = await directoryPluginInvoke(VOICE_ASR_PLUGIN_ID, 'voice.warm', {
      profile,
    })
    rememberSidecarEndpoint(result)
  })().catch(() => {})
}

/**
 * If voice expansion is enabled in saved settings, warm CosyVoice sidecar once (deduped).
 * Does not block callers for the full warm duration — skips when probe reports already warmed.
 */
export async function scheduleVoiceExpansionWarm(
  isPluginDisabled: (id: string) => boolean = () => false,
): Promise<void> {
  if (isPluginDisabled(VOICE_ASR_PLUGIN_ID))
    return
  try {
    const ui = await getPluginSettingsUi(VOICE_ASR_PLUGIN_ID)
    const cfg = ui.config ?? {}
    if (cfg.tts_expansion_enabled !== true)
      return
    const profile
      = typeof cfg.tts_profile === 'string' && cfg.tts_profile.trim()
        ? cfg.tts_profile.trim()
        : DEFAULT_TTS_PROFILE
    const probe = await probeTtsSidecar(profile)
    if (probe)
      rememberSidecarEndpoint(probe)
    if (isSidecarAlreadyWarmed(probe))
      return
    startBackgroundWarm(profile)
    if (cachedSidecarEndpoint)
      return
  }
  catch {
    warmPromise = null
  }
}

/**
 * Resolve sidecar URL for stream fetch without blocking on a long warm/prime cycle.
 * Kicks off background warm and uses probe_tts for the authoritative endpoint.
 */
export async function resolveVoiceSidecarEndpoint(
  profile: string,
  fallbackEndpoint: string,
  isPluginDisabled: (id: string) => boolean = () => false,
): Promise<string> {
  if (cachedSidecarEndpoint)
    return cachedSidecarEndpoint

  if (!isPluginDisabled(VOICE_ASR_PLUGIN_ID)) {
    const probe = await probeTtsSidecar(profile)
    if (probe)
      rememberSidecarEndpoint(probe)
    if (!isSidecarAlreadyWarmed(probe)) {
      try {
        const ui = await getPluginSettingsUi(VOICE_ASR_PLUGIN_ID)
        if (ui.config?.tts_expansion_enabled === true)
          startBackgroundWarm(profile)
      }
      catch {
        /* ignore */
      }
    }
    if (cachedSidecarEndpoint)
      return cachedSidecarEndpoint
  }

  const trimmed = fallbackEndpoint.trim()
  return trimmed || DEFAULT_SIDECAR
}

/** Fire-and-forget startup warm after plugins are ready. */
export function startVoiceExpansionWarmOnStartup(
  isPluginDisabled: (id: string) => boolean = () => false,
): void {
  void scheduleVoiceExpansionWarm(isPluginDisabled)
}

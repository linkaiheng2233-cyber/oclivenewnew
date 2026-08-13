import type { VoiceTtsProfileRouting } from '@oclive/shared/lib/voiceTtsRouting'
import { directoryPluginInvoke, getPluginSettingsUi } from '@oclive/shared/api'
import { invokeWithFriendlyError } from '@oclive/shared/api/helpers'
import { VOICE_ASR_PLUGIN_ID } from '@oclive/shared/lib/voiceAsrEvents'
import {
  explicitVoiceRoleTtsDecision,
  hasExplicitVoiceRoleTtsPolicy,
  normalizeVoiceRoleTtsEnabled,
} from '@oclive/shared/lib/voiceRolePolicy'

const DEFAULT_TTS_PROFILE = 'bundled-cosyvoice2-zh'

export interface VoiceRuntimeConfig {
  tts_expansion_enabled: boolean
  auto_tts: boolean
  role_tts_enabled: Record<string, true>
  role_tts_policy_explicit: boolean
  tts_profile: string
  tts_engine: string
  director_profile: string
  synth_provider: string
  local_synth_endpoint: string
}

let cachedConfig: VoiceRuntimeConfig | null = null
let cachedConfigAt = 0
let loadingConfig: Promise<VoiceRuntimeConfig | null> | null = null
let loadingTtsProfiles: Promise<Map<string, VoiceTtsProfileRouting>> | null = null
let configRevision = 0
const CONFIG_TTL_MS = 30_000

const rolePathCache = new Map<string, string>()
export const roleVoiceProfileConfiguredCache = new Map<string, boolean>()
export const directiveCache = new Map<string, Record<string, unknown>>()
export const directivePending = new Map<string, Promise<Record<string, unknown> | undefined>>()

let cachedTtsProfiles: Map<string, VoiceTtsProfileRouting> | null = null

export function directiveCacheKey(roleId: string, director: string, emotion: string): string {
  return `${roleId}|${director}|${emotion}`
}

export function invalidateVoiceRuntimeConfig(): void {
  configRevision += 1
  cachedConfig = null
  cachedConfigAt = 0
  loadingConfig = null
  cachedTtsProfiles = null
  loadingTtsProfiles = null
  rolePathCache.clear()
  roleVoiceProfileConfiguredCache.clear()
}

export async function loadTtsProfiles(): Promise<Map<string, VoiceTtsProfileRouting>> {
  if (cachedTtsProfiles)
    return cachedTtsProfiles
  if (loadingTtsProfiles)
    return loadingTtsProfiles
  const revision = configRevision
  const promise = (async (): Promise<Map<string, VoiceTtsProfileRouting>> => {
    try {
      const list = (await directoryPluginInvoke(
        VOICE_ASR_PLUGIN_ID,
        'voice.list_profiles',
        {},
      )) as {
        profiles?: Array<{
          id: string
          engine?: string
          synth_provider?: string
          sidecar_endpoint?: string
        }>
      }
      const map = new Map<string, VoiceTtsProfileRouting>()
      for (const row of list.profiles || []) {
        if (row.id) {
          map.set(row.id, {
            engine: row.engine,
            synth_provider: row.synth_provider,
            sidecar_endpoint: row.sidecar_endpoint,
          })
        }
      }
      if (revision === configRevision)
        cachedTtsProfiles = map
      return map
    }
    catch {
      return new Map()
    }
  })()
  loadingTtsProfiles = promise
  try {
    return await promise
  }
  finally {
    if (loadingTtsProfiles === promise)
      loadingTtsProfiles = null
  }
}

export async function loadVoiceRuntimeConfig(
  isPluginDisabled: (id: string) => boolean,
): Promise<VoiceRuntimeConfig | null> {
  if (isPluginDisabled(VOICE_ASR_PLUGIN_ID))
    return null
  const now = Date.now()
  if (cachedConfig && now - cachedConfigAt < CONFIG_TTL_MS)
    return cachedConfig
  if (loadingConfig)
    return loadingConfig
  const revision = configRevision
  const promise = (async (): Promise<VoiceRuntimeConfig | null> => {
    try {
      const [ui, profiles] = await Promise.all([
        getPluginSettingsUi(VOICE_ASR_PLUGIN_ID),
        loadTtsProfiles(),
      ])
      const cfg = ui.config ?? {}
      const ttsProfile
        = typeof cfg.tts_profile === 'string' && cfg.tts_profile.trim()
          ? cfg.tts_profile.trim()
          : DEFAULT_TTS_PROFILE
      const loaded: VoiceRuntimeConfig = {
        tts_expansion_enabled: cfg.tts_expansion_enabled === true,
        auto_tts: cfg.auto_tts === true,
        role_tts_enabled: normalizeVoiceRoleTtsEnabled(cfg.role_tts_enabled),
        role_tts_policy_explicit: hasExplicitVoiceRoleTtsPolicy(cfg),
        tts_profile: ttsProfile,
        tts_engine: profiles.get(ttsProfile)?.engine || 'cosyvoice2',
        director_profile:
          typeof cfg.director_profile === 'string'
            ? cfg.director_profile.trim() || 'none'
            : 'rules-v1',
        synth_provider:
          typeof cfg.synth_provider === 'string' ? cfg.synth_provider.trim() : 'bundled',
        local_synth_endpoint:
          typeof cfg.local_synth_endpoint === 'string'
            ? cfg.local_synth_endpoint.trim()
            : '',
      }
      if (revision === configRevision) {
        cachedConfig = loaded
        cachedConfigAt = Date.now()
      }
      return loaded
    }
    catch {
      return null
    }
  })()
  loadingConfig = promise
  try {
    return await promise
  }
  finally {
    if (loadingConfig === promise)
      loadingConfig = null
  }
}

export async function resolveRolePackPath(roleId: string): Promise<string> {
  const rid = roleId.trim()
  if (!rid)
    return ''
  const cached = rolePathCache.get(rid)
  if (cached !== undefined)
    return cached
  try {
    const path = (await invokeWithFriendlyError<string>('get_role_pack_path', {
      roleId: rid,
    })).trim()
    rolePathCache.set(rid, path)
    return path
  }
  catch {
    rolePathCache.set(rid, '')
    return ''
  }
}

async function roleHasVoiceProfile(roleId: string): Promise<boolean> {
  const rid = roleId.trim()
  if (!rid)
    return false
  const cached = roleVoiceProfileConfiguredCache.get(rid)
  if (cached !== undefined)
    return cached
  const rolePath = await resolveRolePackPath(rid)
  if (!rolePath) {
    roleVoiceProfileConfiguredCache.set(rid, false)
    return false
  }
  try {
    const result = (await directoryPluginInvoke(
      VOICE_ASR_PLUGIN_ID,
      'voice.read_role_profile',
      { role_path: rolePath },
    )) as { profile?: unknown }
    const configured = typeof result.profile === 'object' && result.profile !== null
    roleVoiceProfileConfiguredCache.set(rid, configured)
    return configured
  }
  catch {
    roleVoiceProfileConfiguredCache.set(rid, false)
    return false
  }
}

export async function canAutoSpeakRole(
  cfg: VoiceRuntimeConfig | null,
  roleId: string,
): Promise<boolean> {
  if (!cfg?.tts_expansion_enabled || !cfg.auto_tts)
    return false
  const rid = roleId.trim()
  if (!rid)
    return false
  if (cfg.role_tts_policy_explicit) {
    const enabled = explicitVoiceRoleTtsDecision(
      { role_tts_enabled: cfg.role_tts_enabled },
      rid,
    ) === true
    return enabled && await roleHasVoiceProfile(rid)
  }
  // Compatibility for configs saved before the role map existed: only packs
  // that actually contain voice_profile.json remain eligible.
  return roleHasVoiceProfile(rid)
}

export async function prefetchVoiceDirective(
  roleId: string,
  director: string,
  emotion: string,
): Promise<Record<string, unknown> | undefined> {
  const cacheKey = directiveCacheKey(roleId, director, emotion)
  const cached = directiveCache.get(cacheKey)
  if (cached)
    return cached
  const pending = directivePending.get(cacheKey)
  if (pending)
    return pending
  const promise = (async () => {
    const rolePath = roleId ? await resolveRolePackPath(roleId) : ''
    try {
      const built = (await directoryPluginInvoke(
        VOICE_ASR_PLUGIN_ID,
        'voice.build_directive',
        {
          profile: director,
          bot_emotion: emotion,
          role_path: rolePath,
        },
      )) as { ok?: boolean, directive?: Record<string, unknown> }
      return built.ok ? built.directive : undefined
    }
    catch {
      return undefined
    }
  })()
  directivePending.set(cacheKey, promise)
  try {
    const directive = await promise
    if (directivePending.get(cacheKey) === promise && directive)
      directiveCache.set(cacheKey, directive)
    return directive
  }
  finally {
    if (directivePending.get(cacheKey) === promise)
      directivePending.delete(cacheKey)
  }
}

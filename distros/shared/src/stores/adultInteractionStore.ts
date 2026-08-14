import type {
  AdultInteractionAction,
  AdultInteractionRequest,
  AdultInteractionState,
} from '@oclive/shared/api/chat'
import { defineStore } from 'pinia'

const STORAGE_KEY = 'oclive-chat-pro-adult-settings-v1'

export const ADULT_PACING_INTERVAL_DEFAULT_MS = 4_000
export const ADULT_PACING_INTERVAL_MIN_MS = 500
export const ADULT_PACING_INTERVAL_MAX_MS = 60_000
export const ADULT_BACKGROUND_QUEUE_CAP_DEFAULT = 2
export const ADULT_BACKGROUND_QUEUE_CAP_MIN = 1
export const ADULT_BACKGROUND_QUEUE_CAP_MAX = 8

export interface AdultSessionState {
  active: boolean
  voiceTextOnly: boolean
  updatedAt: number
  generationId?: string
  /** Stored with queued generations so they remain cancellable after an app restart. */
  roleId?: string
  sceneId?: string
}

interface AdultPersistedState {
  confirmedAdult: boolean
  globalEnabled: boolean
  roleEnabled: Record<string, boolean>
  pacingOverrideEnabled: boolean
  pacingIntervalMs: number
  backgroundQueueEnabled: boolean
  backgroundQueueCap: number
  backgroundQueueWarningAccepted: boolean
  sessions: Record<string, AdultSessionState>
}

function defaults(): AdultPersistedState {
  return {
    confirmedAdult: false,
    globalEnabled: false,
    roleEnabled: {},
    pacingOverrideEnabled: false,
    pacingIntervalMs: ADULT_PACING_INTERVAL_DEFAULT_MS,
    backgroundQueueEnabled: false,
    backgroundQueueCap: ADULT_BACKGROUND_QUEUE_CAP_DEFAULT,
    backgroundQueueWarningAccepted: false,
    sessions: {},
  }
}

function integerInRange(
  value: unknown,
  min: number,
  max: number,
): number | undefined {
  const n = Number(value)
  return Number.isSafeInteger(n) && n >= min && n <= max ? n : undefined
}

function boundedInteger(
  value: unknown,
  fallback: number,
  min: number,
  max: number,
): number {
  const n = Number(value)
  if (!Number.isSafeInteger(n))
    return fallback
  return Math.min(max, Math.max(min, n))
}

export function boundedAdultPacingInterval(value: unknown): number {
  return boundedInteger(
    value,
    ADULT_PACING_INTERVAL_DEFAULT_MS,
    ADULT_PACING_INTERVAL_MIN_MS,
    ADULT_PACING_INTERVAL_MAX_MS,
  )
}

export function boundedAdultBackgroundQueueCap(value: unknown): number {
  return boundedInteger(
    value,
    ADULT_BACKGROUND_QUEUE_CAP_DEFAULT,
    ADULT_BACKGROUND_QUEUE_CAP_MIN,
    ADULT_BACKGROUND_QUEUE_CAP_MAX,
  )
}

function readPersisted(): AdultPersistedState {
  const base = defaults()
  if (typeof localStorage === 'undefined')
    return base
  try {
    const parsed = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? '{}') as Partial<AdultPersistedState>
    const pacingIntervalMs = boundedAdultPacingInterval(parsed.pacingIntervalMs)
    const backgroundQueueCap = boundedAdultBackgroundQueueCap(parsed.backgroundQueueCap)
    if (
      parsed.pacingIntervalMs !== undefined
      && Number(parsed.pacingIntervalMs) !== pacingIntervalMs
    ) {
      console.warn('[adult-settings] persisted pacing interval was clamped', {
        previous: parsed.pacingIntervalMs,
        bounded: pacingIntervalMs,
      })
    }
    if (
      parsed.backgroundQueueCap !== undefined
      && Number(parsed.backgroundQueueCap) !== backgroundQueueCap
    ) {
      console.warn('[adult-settings] persisted queue cap was clamped', {
        previous: parsed.backgroundQueueCap,
        bounded: backgroundQueueCap,
      })
    }
    return {
      confirmedAdult: parsed.confirmedAdult === true,
      globalEnabled: parsed.globalEnabled === true && parsed.confirmedAdult === true,
      roleEnabled: parsed.roleEnabled && typeof parsed.roleEnabled === 'object'
        ? { ...parsed.roleEnabled }
        : {},
      pacingOverrideEnabled: parsed.pacingOverrideEnabled === true,
      pacingIntervalMs,
      backgroundQueueEnabled: parsed.backgroundQueueEnabled === true,
      backgroundQueueCap,
      backgroundQueueWarningAccepted: parsed.backgroundQueueWarningAccepted === true,
      sessions: parsed.sessions && typeof parsed.sessions === 'object'
        ? { ...parsed.sessions }
        : {},
    }
  }
  catch {
    return base
  }
}

function sessionKey(roleId: string, sceneId: string): string {
  return `${roleId.trim()}:${sceneId.trim() || 'default'}`
}

export const useAdultInteractionStore = defineStore('adult-interaction', {
  state: readPersisted,
  getters: {
    roleIsEnabled: state => (roleId: string): boolean => state.roleEnabled[roleId] === true,
    sessionFor: state => (roleId: string, sceneId: string): AdultSessionState =>
      state.sessions[sessionKey(roleId, sceneId)] ?? {
        active: false,
        voiceTextOnly: false,
        updatedAt: 0,
      },
    gatesOpen(): boolean {
      return this.confirmedAdult && this.globalEnabled
    },
  },
  actions: {
    persist() {
      if (typeof localStorage === 'undefined')
        return
      localStorage.setItem(STORAGE_KEY, JSON.stringify({
        confirmedAdult: this.confirmedAdult,
        globalEnabled: this.globalEnabled,
        roleEnabled: this.roleEnabled,
        pacingOverrideEnabled: this.pacingOverrideEnabled,
        pacingIntervalMs: boundedAdultPacingInterval(this.pacingIntervalMs),
        backgroundQueueEnabled: this.backgroundQueueEnabled,
        backgroundQueueCap: boundedAdultBackgroundQueueCap(this.backgroundQueueCap),
        backgroundQueueWarningAccepted: this.backgroundQueueWarningAccepted,
        sessions: this.sessions,
      } satisfies AdultPersistedState))
    },
    confirmAndEnableGlobal() {
      this.confirmedAdult = true
      this.globalEnabled = true
      this.persist()
    },
    setGlobalEnabled(enabled: boolean) {
      if (enabled && !this.confirmedAdult)
        return false
      this.globalEnabled = enabled
      if (!enabled) {
        for (const [key, session] of Object.entries(this.sessions)) {
          if (session.generationId) {
            this.sessions[key] = {
              ...session,
              active: false,
              voiceTextOnly: false,
              updatedAt: Date.now(),
            }
          }
          else {
            delete this.sessions[key]
          }
        }
      }
      this.persist()
      return true
    },
    setRoleEnabled(roleId: string, enabled: boolean) {
      const id = roleId.trim()
      if (!id)
        return
      this.roleEnabled[id] = enabled
      if (!enabled) {
        for (const [key, session] of Object.entries(this.sessions)) {
          if (!key.startsWith(`${id}:`))
            continue
          if (session.generationId) {
            this.sessions[key] = {
              ...session,
              active: false,
              voiceTextOnly: false,
              updatedAt: Date.now(),
            }
          }
          else {
            delete this.sessions[key]
          }
        }
      }
      this.persist()
    },
    updateSession(
      roleId: string,
      sceneId: string,
      interactionState: AdultInteractionState,
    ) {
      const key = sessionKey(roleId, sceneId)
      if (interactionState === 'active') {
        const previous = this.sessions[key]
        this.sessions[key] = {
          active: true,
          voiceTextOnly: previous?.voiceTextOnly ?? false,
          updatedAt: Date.now(),
          roleId: roleId.trim(),
          sceneId: sceneId.trim() || 'default',
          ...(previous?.generationId
            ? { generationId: previous.generationId }
            : {}),
        }
      }
      else {
        this.clearSession(roleId, sceneId)
      }
      this.persist()
    },
    clearSession(roleId: string, sceneId: string) {
      const key = sessionKey(roleId, sceneId)
      const current = this.sessions[key]
      if (current?.generationId) {
        this.sessions[key] = {
          ...current,
          active: false,
          voiceTextOnly: false,
          updatedAt: Date.now(),
        }
      }
      else {
        delete this.sessions[key]
      }
      this.persist()
    },
    markVoiceTextOnly(roleId: string, sceneId: string) {
      const key = sessionKey(roleId, sceneId)
      const current = this.sessions[key]
      if (!current?.active)
        return
      this.sessions[key] = {
        ...current,
        voiceTextOnly: true,
        updatedAt: Date.now(),
      }
      this.persist()
    },
    requestFor(
      roleId: string,
      sceneId: string,
      action: AdultInteractionAction = 'message',
    ): AdultInteractionRequest | undefined {
      if (!this.confirmedAdult || !this.globalEnabled || !this.roleIsEnabled(roleId))
        return undefined
      return {
        confirmed_adult: true,
        global_enabled: true,
        role_enabled: true,
        interaction_active: this.sessionFor(roleId, sceneId).active,
        action,
      }
    },
    setPacingOverride(enabled: boolean, intervalMs: number): boolean {
      const accepted = integerInRange(
        intervalMs,
        ADULT_PACING_INTERVAL_MIN_MS,
        ADULT_PACING_INTERVAL_MAX_MS,
      )
      if (accepted === undefined)
        return false
      this.pacingOverrideEnabled = enabled
      this.pacingIntervalMs = accepted
      this.persist()
      return true
    },
    setBackgroundQueue(
      enabled: boolean,
      cap?: number,
      warningAccepted?: boolean,
    ): boolean {
      const acceptedCap = integerInRange(
        cap ?? this.backgroundQueueCap,
        ADULT_BACKGROUND_QUEUE_CAP_MIN,
        ADULT_BACKGROUND_QUEUE_CAP_MAX,
      )
      if (acceptedCap === undefined)
        return false
      this.backgroundQueueEnabled = enabled
      this.backgroundQueueCap = acceptedCap
      this.backgroundQueueWarningAccepted
        = warningAccepted ?? this.backgroundQueueWarningAccepted
      this.persist()
      return true
    },
    resetAdultSettings() {
      const reset = defaults()
      const cancellationTombstones = Object.fromEntries(
        Object.entries(this.sessions)
          .filter(([, session]) => Boolean(session.generationId))
          .map(([key, session]) => [key, {
            ...session,
            active: false,
            voiceTextOnly: false,
            updatedAt: Date.now(),
          }]),
      )
      this.confirmedAdult = reset.confirmedAdult
      this.globalEnabled = reset.globalEnabled
      this.roleEnabled = reset.roleEnabled
      this.pacingOverrideEnabled = reset.pacingOverrideEnabled
      this.pacingIntervalMs = reset.pacingIntervalMs
      this.backgroundQueueEnabled = reset.backgroundQueueEnabled
      this.backgroundQueueCap = reset.backgroundQueueCap
      this.backgroundQueueWarningAccepted = reset.backgroundQueueWarningAccepted
      this.sessions = cancellationTombstones
      this.persist()
    },
    setSessionGeneration(roleId: string, sceneId: string, generationId?: string) {
      const key = sessionKey(roleId, sceneId)
      const current = this.sessions[key]
      if (!current && !generationId)
        return
      this.sessions[key] = {
        ...current,
        active: current?.active ?? false,
        voiceTextOnly: current?.voiceTextOnly ?? false,
        roleId: roleId.trim(),
        sceneId: sceneId.trim() || 'default',
        ...(generationId ? { generationId } : {}),
        updatedAt: Date.now(),
      }
      if (!generationId) {
        delete this.sessions[key]!.generationId
        if (!this.sessions[key]!.active)
          delete this.sessions[key]
      }
      this.persist()
    },
  },
})

export { sessionKey as adultSessionKey }

import type {
  AdultInteractionAction,
  AdultInteractionRequest,
  AdultInteractionState,
} from '@oclive/shared/api/chat'
import { defineStore } from 'pinia'

const STORAGE_KEY = 'oclive-chat-pro-adult-settings-v1'

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
    pacingIntervalMs: 4_000,
    backgroundQueueEnabled: false,
    backgroundQueueCap: 2,
    backgroundQueueWarningAccepted: false,
    sessions: {},
  }
}

function positiveInteger(value: unknown, fallback: number): number {
  const n = Number(value)
  return Number.isSafeInteger(n) && n > 0 ? n : fallback
}

function readPersisted(): AdultPersistedState {
  const base = defaults()
  if (typeof localStorage === 'undefined')
    return base
  try {
    const parsed = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? '{}') as Partial<AdultPersistedState>
    return {
      confirmedAdult: parsed.confirmedAdult === true,
      globalEnabled: parsed.globalEnabled === true && parsed.confirmedAdult === true,
      roleEnabled: parsed.roleEnabled && typeof parsed.roleEnabled === 'object'
        ? { ...parsed.roleEnabled }
        : {},
      pacingOverrideEnabled: parsed.pacingOverrideEnabled === true,
      pacingIntervalMs: positiveInteger(parsed.pacingIntervalMs, base.pacingIntervalMs),
      backgroundQueueEnabled: parsed.backgroundQueueEnabled === true,
      backgroundQueueCap: positiveInteger(
        parsed.backgroundQueueCap,
        base.backgroundQueueCap,
      ),
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
        pacingIntervalMs: positiveInteger(this.pacingIntervalMs, 4_000),
        backgroundQueueEnabled: this.backgroundQueueEnabled,
        backgroundQueueCap: positiveInteger(this.backgroundQueueCap, 2),
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
    setPacingOverride(enabled: boolean, intervalMs: number) {
      this.pacingOverrideEnabled = enabled
      this.pacingIntervalMs = positiveInteger(intervalMs, this.pacingIntervalMs)
      this.persist()
    },
    setBackgroundQueue(
      enabled: boolean,
      cap: number = this.backgroundQueueCap,
      warningAccepted: boolean = this.backgroundQueueWarningAccepted,
    ) {
      this.backgroundQueueEnabled = enabled
      this.backgroundQueueCap = positiveInteger(cap, this.backgroundQueueCap)
      this.backgroundQueueWarningAccepted = warningAccepted
      this.persist()
    },
    setSessionGeneration(roleId: string, sceneId: string, generationId?: string) {
      const key = sessionKey(roleId, sceneId)
      const current = this.sessions[key]
      if (!current && !generationId)
        return
      this.sessions[key] = {
        active: current?.active ?? false,
        voiceTextOnly: current?.voiceTextOnly ?? false,
        ...current,
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

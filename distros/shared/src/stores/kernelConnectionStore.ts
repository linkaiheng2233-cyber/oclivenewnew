import type { KernelConnectionStatus } from '@oclive/shared/api/kernel'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { defineStore } from 'pinia'
import { getKernelConnectionStatus, reconnectKernel } from '@oclive/shared/api/kernel'
import { i18n } from '@oclive/shared/i18n'
import { kernelProfileDetailKey, kernelStatusLabelKey } from '@oclive/shared/lib/kernelProfileUx'
import { useChatStore } from './chatStore'

export type KernelPhase = 'checking' | 'ready'

export interface KernelDisplay {
  labelKey: string
  detailKey: string | null
  clickable: boolean
  ok: boolean
  checking: boolean
}

let unlistenFns: UnlistenFn[] = []
let eventsBound = false

export const useKernelConnectionStore = defineStore('kernelConnection', {
  state: () => ({
    phase: 'checking' as KernelPhase,
    status: null as KernelConnectionStatus | null,
    lastError: null as string | null,
    busy: false,
    initialized: false,
    wasHealthy: false as boolean,
  }),

  getters: {
    display(state): KernelDisplay {
      if (state.phase === 'checking') {
        return {
          labelKey: 'kernel.status.checking',
          detailKey: null,
          clickable: false,
          ok: false,
          checking: true,
        }
      }
      if (state.busy || state.status?.mode === 'reconnecting') {
        return {
          labelKey: 'kernel.status.reconnecting',
          detailKey: null,
          clickable: false,
          ok: false,
          checking: false,
        }
      }
      if (!state.status?.healthy) {
        if (state.lastError) {
          return {
            labelKey: 'kernel.status.offlineRetryFailed',
            detailKey: null,
            clickable: true,
            ok: false,
            checking: false,
          }
        }
        return {
          labelKey: 'kernel.status.offlineTapReconnect',
          detailKey: null,
          clickable: true,
          ok: false,
          checking: false,
        }
      }
      return {
        labelKey: kernelStatusLabelKey(state.status),
        detailKey: kernelProfileDetailKey(state.status),
        clickable: false,
        ok: true,
        checking: false,
      }
    },

    disabled(state): boolean {
      return (
        state.busy
        || state.phase === 'checking'
        || Boolean(state.status?.healthy)
      )
    },

    canReconnect(state): boolean {
      return (
        state.phase === 'ready'
        && !state.busy
        && !state.status?.healthy
      )
    },

    alreadyConnected(state): boolean {
      return state.phase === 'ready' && Boolean(state.status?.healthy)
    },
  },

  actions: {
    applyStatus(status: KernelConnectionStatus) {
      const prevHealthy = this.wasHealthy
      this.status = status
      this.wasHealthy = status.healthy
      if (prevHealthy && !status.healthy) {
        const chatStore = useChatStore()
        chatStore.addSystemMessage(i18n.global.t('kernel.chat.disconnected'))
      }
    },

    async refresh() {
      try {
        const status = await getKernelConnectionStatus()
        this.applyStatus(status)
        if (status.healthy) {
          this.lastError = null
        }
      }
      catch (err) {
        console.warn('[kernelConnection] refresh failed', err)
        this.status = null
        this.wasHealthy = false
      }
    },

    async init() {
      if (this.initialized) {
        return
      }
      this.initialized = true
      this.phase = 'checking'
      await this.refresh()
      this.phase = 'ready'
      await this.bindEvents()
    },

    async bindEvents() {
      if (eventsBound) {
        return
      }
      eventsBound = true
      const handler = (payload: KernelConnectionStatus) => {
        this.applyStatus(payload)
        if (payload.healthy) {
          this.lastError = null
        }
      }
      for (const event of [
        'kernel:status_changed',
        'kernel:upstream_lost',
        'kernel:reconnected',
      ] as const) {
        const unlisten = await listen<KernelConnectionStatus>(event, (e) => {
          handler(e.payload)
        })
        unlistenFns.push(unlisten)
      }
    },

    teardownEvents() {
      for (const fn of unlistenFns) {
        fn()
      }
      unlistenFns = []
      eventsBound = false
    },

    async reconnect() {
      if (this.busy) {
        return this.status
      }
      if (this.phase === 'ready' && this.status?.healthy) {
        return this.status
      }
      this.busy = true
      this.lastError = null
      try {
        const status = await reconnectKernel()
        this.applyStatus(status)
        return status
      }
      catch (err) {
        this.lastError = err instanceof Error ? err.message : String(err)
        await this.refresh()
        throw err
      }
      finally {
        this.busy = false
      }
    },
  },
})

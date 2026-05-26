import { defineStore } from 'pinia'

function readPersistedUiState(): {
  sceneId: string
  connectivityBanner: null | {
    kind: 'plugin_index_offline'
    detail?: string
  }
} {
  const base = {
    sceneId: 'home',
    connectivityBanner: null as null | {
      kind: 'plugin_index_offline'
      detail?: string
    },
  }
  try {
    const raw = localStorage.getItem('ui')
    if (!raw)
      return base
    const parsed = JSON.parse(raw) as Record<string, unknown>
    if (typeof parsed.sceneId === 'string' && parsed.sceneId.trim()) {
      base.sceneId = parsed.sceneId
    }
  }
  catch {
    /* ignore */
  }
  return base
}

export const useUiStore = defineStore('ui', {
  state: () => readPersistedUiState(),
  actions: {
    setScene(sceneId: string) {
      this.sceneId = sceneId
    },
    setPluginIndexOfflineBanner(detail?: string) {
      this.connectivityBanner = {
        kind: 'plugin_index_offline',
        detail: detail?.trim() || undefined,
      }
    },
    clearPluginIndexConnectivityBanner() {
      if (this.connectivityBanner?.kind === 'plugin_index_offline') {
        this.connectivityBanner = null
      }
    },
    dismissConnectivityBanner() {
      this.connectivityBanner = null
    },
  },
  persist: {
    pick: ['sceneId'],
  },
})

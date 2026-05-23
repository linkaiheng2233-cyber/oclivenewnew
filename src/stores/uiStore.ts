import { defineStore } from 'pinia'

function readPersistedUiState(): {
  sceneId: string
  advancedPluginManagement: boolean
  connectivityBanner: null | {
    kind: 'plugin_index_offline'
    detail?: string
  }
} {
  const base = {
    sceneId: 'home',
    advancedPluginManagement: false,
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
    if (typeof parsed.advancedPluginManagement === 'boolean') {
      base.advancedPluginManagement = parsed.advancedPluginManagement
    }
    else if (parsed.experimentalPluginManagerV2 === true) {
      base.advancedPluginManagement = true
    }
  }
  catch {
    /* ignore */
  }
  return base
}

export const useUiStore = defineStore('ui', {
  state: () => readPersistedUiState(),
  getters: {
    /** @deprecated 使用 `advancedPluginManagement` */
    experimentalPluginManagerV2(state): boolean {
      return state.advancedPluginManagement
    },
  },
  actions: {
    setScene(sceneId: string) {
      this.sceneId = sceneId
    },
    setAdvancedPluginManagement(enabled: boolean) {
      this.advancedPluginManagement = enabled
    },
    /** @deprecated 迁移自 experimentalPluginManagerV2 */
    setExperimentalPluginManagerV2(enabled: boolean) {
      this.setAdvancedPluginManagement(enabled)
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
    pick: ['sceneId', 'advancedPluginManagement'],
  },
})

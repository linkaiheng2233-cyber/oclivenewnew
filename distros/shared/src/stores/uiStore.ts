import { defineStore } from 'pinia'

export const useUiStore = defineStore('ui', {
  state: () => ({
    sceneId: 'home',
    connectivityBanner: null as null | {
      kind: 'plugin_index_offline'
      detail?: string
    },
  }),
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

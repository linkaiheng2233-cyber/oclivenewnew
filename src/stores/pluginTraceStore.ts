import { defineStore } from 'pinia'
import { usePluginMarketStore } from './pluginMarketStore'
import { usePluginStore } from './pluginStore'
import { traceActions, traceState } from './plugin/traceSlice'
import type { PluginPanelMainTab } from './plugin/constants'
import type { TraceSliceStore } from './plugin/traceSlice'

export const usePluginTraceStore = defineStore('pluginTrace', {
  state: () => traceState(),
  actions: {
    ...traceActions,
    async openPanel(this: TraceSliceStore, tab?: PluginPanelMainTab) {
      if (tab) {
        this.panelMainTab = tab
      }
      this.panelVisible = true
      usePluginMarketStore().closeMarketPanel()
      await usePluginStore().refresh()
    },
    openMarketPanel(this: TraceSliceStore) {
      const market = usePluginMarketStore()
      market.openMarketPanel()
      this.panelVisible = false
    },
    closeMarketPanel() {
      usePluginMarketStore().closeMarketPanel()
    },
    requestOpenSimplePluginManager(this: TraceSliceStore) {
      this.closePanel()
      usePluginMarketStore().closeMarketPanel()
      this.simpleManagerOpenNonce += 1
    },
    requestFocusInstalledPlugin(this: TraceSliceStore, pluginId: string) {
      const id = pluginId.trim()
      if (!id)
        return
      this.focusPluginId = id
      usePluginMarketStore().closeMarketPanel()
      this.panelVisible = true
    },
    requestFocusArchSlot(
      this: TraceSliceStore,
      slotKey: string,
      tab: PluginPanelMainTab = 'graph',
    ) {
      const key = slotKey.trim()
      if (!key)
        return
      this.focusSlotKey = key
      this.panelMainTab = tab
      this.panelVisible = true
      usePluginMarketStore().closeMarketPanel()
    },
    togglePanel(this: TraceSliceStore) {
      if (this.panelVisible) {
        this.closePanel()
      }
      else {
        void this.openPanel()
      }
    },
  },
})

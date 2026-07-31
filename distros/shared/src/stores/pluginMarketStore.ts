import type { MarketSliceStore } from './plugin/marketSlice'
import { ensurePluginWorkbenchI18n } from '@oclive/shared/i18n/loadPluginWorkbench'
import { defineStore } from 'pinia'
import { marketActions, marketState } from './plugin/marketSlice'

export const usePluginMarketStore = defineStore('pluginMarket', {
  state: () => ({
    ...marketState(),
    marketPanelVisible: false,
  }),
  actions: {
    ...marketActions,
    async openMarketPanel(this: MarketSliceStore & { marketPanelVisible: boolean }) {
      await ensurePluginWorkbenchI18n()
      this.marketPanelVisible = true
      this.pendingGitShareUrl = null
      if (!this.shareCatalogUrl.trim() && !this.pluginMarketSnapshot) {
        void this.loadCachedPluginMarket()
      }
    },
    closeMarketPanel(this: { marketPanelVisible: boolean }) {
      this.marketPanelVisible = false
    },
  },
})

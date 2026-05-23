import { defineStore } from 'pinia'
import { marketActions, marketState } from './plugin/marketSlice'
import type { MarketSliceStore } from './plugin/marketSlice'

export const usePluginMarketStore = defineStore('pluginMarket', {
  state: () => ({
    ...marketState(),
    marketPanelVisible: false,
  }),
  actions: {
    ...marketActions,
    openMarketPanel(this: MarketSliceStore & { marketPanelVisible: boolean }) {
      this.marketPanelVisible = true
      void this.loadCachedPluginMarket()
    },
    closeMarketPanel(this: { marketPanelVisible: boolean }) {
      this.marketPanelVisible = false
    },
  },
})

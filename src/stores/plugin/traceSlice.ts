import type { PluginPanelMainTab } from './constants'

export function traceState() {
  return {
    panelVisible: false,
    marketPanelVisible: false,
    /** 递增后由 App 打开极简插件管理（市场等入口） */
    simpleManagerOpenNonce: 0,
    panelMainTab: 'graph' as PluginPanelMainTab,
    /** 架构图等入口请求聚焦的已安装插件 id */
    focusPluginId: null as string | null,
    /** V2 / 外部入口请求在架构图中高亮的 slot_registry 实例键 */
    focusSlotKey: null as string | null,
  }
}

export const traceActions = {
  async openPanel(this: TraceSliceStore, tab?: PluginPanelMainTab) {
    if (tab) {
      this.panelMainTab = tab
    }
    this.panelVisible = true
    this.marketPanelVisible = false
    await this.refresh()
  },
  async openMarketPanel(this: TraceSliceStore) {
    this.marketPanelVisible = true
    this.panelVisible = false
    await this.loadCachedPluginMarket()
  },
  closeMarketPanel(this: TraceSliceStore) {
    this.marketPanelVisible = false
  },
  requestOpenSimplePluginManager(this: TraceSliceStore) {
    this.closePanel()
    this.marketPanelVisible = false
    this.simpleManagerOpenNonce += 1
  },
  requestFocusInstalledPlugin(this: TraceSliceStore, pluginId: string) {
    const id = pluginId.trim()
    if (!id)
      return
    this.focusPluginId = id
    this.marketPanelVisible = false
    this.panelVisible = true
  },
  clearFocusInstalledPlugin(this: TraceSliceStore) {
    this.focusPluginId = null
  },
  requestFocusArchSlot(this: TraceSliceStore, slotKey: string, tab: PluginPanelMainTab = 'graph') {
    const key = slotKey.trim()
    if (!key)
      return
    this.focusSlotKey = key
    this.panelMainTab = tab
    this.panelVisible = true
    this.marketPanelVisible = false
  },
  clearFocusArchSlot(this: TraceSliceStore) {
    this.focusSlotKey = null
  },
  closePanel(this: TraceSliceStore) {
    this.panelVisible = false
    this.clearFocusInstalledPlugin()
  },
  togglePanel(this: TraceSliceStore) {
    if (this.panelVisible) {
      this.closePanel()
    }
    else {
      void this.openPanel()
    }
  },
}

/** Minimal typing for cross-slice `this` in trace actions. */
export interface TraceSliceStore {
  panelVisible: boolean
  marketPanelVisible: boolean
  simpleManagerOpenNonce: number
  panelMainTab: PluginPanelMainTab
  focusPluginId: string | null
  focusSlotKey: string | null
  refresh(): Promise<void>
  loadCachedPluginMarket(): Promise<void>
  closePanel(): void
  clearFocusInstalledPlugin(): void
  openPanel(tab?: PluginPanelMainTab): Promise<void>
}

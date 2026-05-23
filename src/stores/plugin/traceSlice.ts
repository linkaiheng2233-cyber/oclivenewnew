import type { PluginPanelMainTab } from './constants'

export function traceState() {
  return {
    panelVisible: false,
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
  clearFocusInstalledPlugin(this: TraceSliceStore) {
    this.focusPluginId = null
  },
  clearFocusArchSlot(this: TraceSliceStore) {
    this.focusSlotKey = null
  },
  closePanel(this: TraceSliceStore) {
    this.panelVisible = false
    this.clearFocusInstalledPlugin()
  },
}

/** Minimal typing for cross-slice `this` in trace actions. */
export interface TraceSliceStore {
  panelVisible: boolean
  simpleManagerOpenNonce: number
  panelMainTab: PluginPanelMainTab
  focusPluginId: string | null
  focusSlotKey: string | null
  clearFocusInstalledPlugin(): void
  openPanel(tab?: PluginPanelMainTab): Promise<void>
}

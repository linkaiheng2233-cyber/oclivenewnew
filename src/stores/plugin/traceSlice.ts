import type { PluginPanelMainTab } from './constants'

export function traceState() {
  return {
    panelVisible: false,
    /** Incremented to open minimal plugin manager from App (market and other entry points) */
    simpleManagerOpenNonce: 0,
    panelMainTab: 'graph' as PluginPanelMainTab,
    /** Installed plugin id to focus from architecture graph and similar entry points */
    focusPluginId: null as string | null,
    /** V2 / external entry: slot_registry instance key to highlight in architecture graph */
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

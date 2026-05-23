import type { PluginMarketSnapshotDto } from '../../api'
import {
  batchUpdatePlugins,
  getCachedPluginIndex,
  installPluginFromMarket,
  syncPluginIndexCommand,
  uninstallPluginFromMarket,
  updatePluginFromMarket,
} from '../../api'
import { useUiStore } from '../uiStore'

export function marketState() {
  return {
    /** 最近一次 `get_cached_plugin_index` / `sync_plugin_index_command` 快照 */
    pluginMarketSnapshot: null as PluginMarketSnapshotDto | null,
    pluginMarketSyncing: false,
    pluginMarketError: null as string | null,
  }
}

export const marketActions = {
  async loadCachedPluginMarket(this: MarketSliceStore) {
    this.pluginMarketError = null
    try {
      this.pluginMarketSnapshot = await getCachedPluginIndex()
    }
    catch (e) {
      this.pluginMarketError = e instanceof Error ? e.message : String(e)
    }
  },
  async syncPluginMarket(this: MarketSliceStore, indexUrl?: string | null) {
    this.pluginMarketSyncing = true
    this.pluginMarketError = null
    try {
      this.pluginMarketSnapshot = await syncPluginIndexCommand(
        indexUrl ?? undefined,
      )
      const ui = useUiStore()
      if (
        this.pluginMarketSnapshot?.offlineMode
        && this.pluginMarketSnapshot?.warning
      ) {
        ui.setPluginIndexOfflineBanner(this.pluginMarketSnapshot.warning)
      }
      else {
        ui.clearPluginIndexConnectivityBanner()
      }
      await this.refresh()
    }
    catch (e) {
      this.pluginMarketError = e instanceof Error ? e.message : String(e)
      throw e
    }
    finally {
      this.pluginMarketSyncing = false
    }
  },
  async installFromPluginMarket(this: MarketSliceStore, pluginId: string, gitUrl?: string | null) {
    await installPluginFromMarket(pluginId, gitUrl ?? null)
    await this.refresh()
    this.bootstrapEpoch += 1
  },
  async updateInstalledPluginFromGit(this: MarketSliceStore, pluginId: string) {
    await updatePluginFromMarket(pluginId)
    await this.refresh()
    this.bootstrapEpoch += 1
  },
  async uninstallPluginFromGitIndex(this: MarketSliceStore, pluginId: string) {
    await uninstallPluginFromMarket(pluginId)
    await this.refresh()
    this.bootstrapEpoch += 1
  },
  async batchUpdatePluginsFromGitIndex(this: MarketSliceStore, pluginIds: string[]) {
    await batchUpdatePlugins(pluginIds)
    await this.refresh()
    this.bootstrapEpoch += 1
  },
}

export interface MarketSliceStore {
  pluginMarketSnapshot: PluginMarketSnapshotDto | null
  pluginMarketSyncing: boolean
  pluginMarketError: string | null
  bootstrapEpoch: number
  refresh(): Promise<void>
}

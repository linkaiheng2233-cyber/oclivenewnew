import type { PluginMarketSnapshotDto } from '@oclive/shared/api'
import {
  batchUpdatePlugins,
  getCachedPluginIndex,
  installPluginFromGit,
  installPluginFromMarket,
  syncPluginIndexCommand,
  uninstallPluginFromMarket,
  updatePluginFromMarket,
} from '@oclive/shared/api'
import { classifyPluginShareUrl } from '@oclive/shared/lib/pluginShareUrl'
import { useUiStore } from '../uiStore'
import { usePluginStore } from '../pluginStore'

const SHARE_URL_STORAGE_KEY = 'oclive-plugin-market-share-url'

function readStoredShareUrl(): string {
  if (typeof localStorage === 'undefined') {
    return ''
  }
  return localStorage.getItem(SHARE_URL_STORAGE_KEY)?.trim() ?? ''
}

function writeStoredShareUrl(url: string): void {
  if (typeof localStorage === 'undefined') {
    return
  }
  const t = url.trim()
  if (t) {
    localStorage.setItem(SHARE_URL_STORAGE_KEY, t)
  }
  else {
    localStorage.removeItem(SHARE_URL_STORAGE_KEY)
  }
}

export function marketState() {
  return {
    /** Latest snapshot from `get_cached_plugin_index` / `sync_plugin_index_command` */
    pluginMarketSnapshot: null as PluginMarketSnapshotDto | null,
    pluginMarketSyncing: false,
    pluginMarketError: null as string | null,
    /** Share link input (restores last pasted URL when opening market) */
    shareCatalogUrl: readStoredShareUrl(),
    /** Current git-repo shared plugin (single install; no plugins.json list) */
    pendingGitShareUrl: null as string | null,
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
  async loadFromShareUrl(this: MarketSliceStore, rawUrl: string) {
    const url = rawUrl.trim()
    if (!url) {
      this.pluginMarketError = 'share_url_required'
      throw new Error('share_url_required')
    }
    const kind = classifyPluginShareUrl(url)
    if (kind === 'invalid') {
      this.pluginMarketError = 'share_url_invalid'
      throw new Error('share_url_invalid')
    }
    this.shareCatalogUrl = url
    writeStoredShareUrl(url)
    if (kind === 'git') {
      this.pendingGitShareUrl = url
      this.pluginMarketSnapshot = null
      this.pluginMarketError = null
      return
    }
    this.pendingGitShareUrl = null
    await this.syncPluginMarket(url)
  },
  async installFromGitShare(this: MarketSliceStore, gitUrl: string) {
    const url = gitUrl.trim()
    if (!url) {
      return
    }
    this.pluginMarketSyncing = true
    this.pluginMarketError = null
    try {
      await installPluginFromGit(url)
      this.pendingGitShareUrl = null
      const pluginStore = usePluginStore()
      await pluginStore.refresh()
      pluginStore.bootstrapEpoch += 1
    }
    catch (e) {
      this.pluginMarketError = e instanceof Error ? e.message : String(e)
      throw e
    }
    finally {
      this.pluginMarketSyncing = false
    }
  },
  clearPendingGitShare(this: MarketSliceStore) {
    this.pendingGitShareUrl = null
  },
  async syncPluginMarket(this: MarketSliceStore, indexUrl?: string | null) {
    this.pluginMarketSyncing = true
    this.pluginMarketError = null
    this.pendingGitShareUrl = null
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
      await usePluginStore().refresh()
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
    const pluginStore = usePluginStore()
    await pluginStore.refresh()
    pluginStore.bootstrapEpoch += 1
  },
  async updateInstalledPluginFromGit(this: MarketSliceStore, pluginId: string) {
    await updatePluginFromMarket(pluginId)
    const pluginStore = usePluginStore()
    await pluginStore.refresh()
    pluginStore.bootstrapEpoch += 1
  },
  async uninstallPluginFromGitIndex(this: MarketSliceStore, pluginId: string) {
    await uninstallPluginFromMarket(pluginId)
    const pluginStore = usePluginStore()
    await pluginStore.refresh()
    pluginStore.bootstrapEpoch += 1
  },
  async batchUpdatePluginsFromGitIndex(this: MarketSliceStore, pluginIds: string[]) {
    await batchUpdatePlugins(pluginIds)
    const pluginStore = usePluginStore()
    await pluginStore.refresh()
    pluginStore.bootstrapEpoch += 1
  },
}

export interface MarketSliceStore {
  pluginMarketSnapshot: PluginMarketSnapshotDto | null
  pluginMarketSyncing: boolean
  pluginMarketError: string | null
  shareCatalogUrl: string
  pendingGitShareUrl: string | null
}

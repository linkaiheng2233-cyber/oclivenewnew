import type {
  DirectoryPluginBootstrap,
  DirectoryPluginCatalogEntry,
  PluginUiSlotInfo,
  PluginUpdateInfo,
} from '../../api'
import {
  checkPluginUpdates,
  extractPluginZip,
  getDirectoryPluginBootstrap,
  getDirectoryPluginCatalog,
  getPluginState,
} from '../../api'
import { setHostEventSubscribedEvents } from '../../lib/hostEventBus'
import { useRoleStore } from '../roleStore'
import {
  catalogEqual,
  clonePluginState,
  refreshPromise,
  rolePluginStateEqual,
  setRefreshPromise,
} from './constants'
import type { InstalledSliceStore } from './installedSlice'

export function directoryState() {
  return {
    loading: false,
    error: null as string | null,
    catalog: [] as DirectoryPluginCatalogEntry[],
    /** 目录插件 catalog 预计算：各 slot 对应的非整壳插件 id（已排序）。 */
    catalogCandidatesBySlot: {} as Record<string, string[]>,
    /** 与 `get_directory_plugin_bootstrap.developer_mode` 一致（扫描额外插件根等）。 */
    developerMode: false,
    /** 最近一次 bootstrap 的嵌入插槽列表（与 `get_directory_plugin_bootstrap.uiSlots` 一致）。 */
    bootstrapUiSlots: [] as PluginUiSlotInfo[],
    /** 变更后嵌入插槽组件会重新拉 bootstrap */
    bootstrapEpoch: 0,
    /** `check_plugin_updates` 最近一次结果（按插件 id）。 */
    pluginUpdateById: {} as Record<string, PluginUpdateInfo>,
    pluginUpdatesCheckLoading: false,
    extractingPluginId: null as string | null,
  }
}

export const directoryActions = {
  /** 由 bootstrap DTO 更新宿主事件订阅与开发者模式（插槽与 `refresh` / `sync` 共用）。 */
  applyDirectoryBootstrap(this: DirectorySliceStore, boot: DirectoryPluginBootstrap) {
    setHostEventSubscribedEvents(boot.subscribedHostEvents ?? [])
    this.developerMode = boot.developerMode ?? false
    this.bootstrapUiSlots = boot.uiSlots ?? []
  },
  /** 角色切换或插件启用状态变更后更新宿主事件订阅与开发者模式（不拉 catalog）。 */
  async syncDirectoryPluginBootstrap(this: DirectorySliceStore) {
    const roleId = useRoleStore().currentRoleId
    try {
      const boot = await getDirectoryPluginBootstrap(roleId)
      this.applyDirectoryBootstrap(boot)
    }
    catch (e) {
      this.error = e instanceof Error ? e.message : String(e)
    }
  },
  async refresh(this: DirectorySliceStore) {
    if (refreshPromise) {
      return refreshPromise
    }
    this.loading = true
    this.error = null
    const promise = (async () => {
      try {
        const roleId = useRoleStore().currentRoleId
        const [cat, bundle, boot] = await Promise.all([
          getDirectoryPluginCatalog(),
          getPluginState(roleId),
          getDirectoryPluginBootstrap(roleId),
        ])
        this.pluginStateBundle = {
          role: clonePluginState(bundle.role),
          globalDefaults: clonePluginState(bundle.globalDefaults),
        }
        const st
          = this.persistScope === 'role' ? bundle.role : bundle.globalDefaults
        const nextState = clonePluginState(st)
        if (!catalogEqual(this.catalog, cat)) {
          this.catalog = cat
          this.slotOrderMemoBySlot = {}
          const bySlot: Record<string, string[]> = {}
          for (const p of cat) {
            if (p.isShell)
              continue
            for (const slotName of p.uiSlotNames ?? []) {
              if (!bySlot[slotName])
                bySlot[slotName] = []
              bySlot[slotName].push(p.id)
            }
          }
          for (const slotName of Object.keys(bySlot)) {
            bySlot[slotName].sort()
          }
          this.catalogCandidatesBySlot = bySlot
        }
        if (!rolePluginStateEqual(this.pluginState, nextState)) {
          this.pluginState = nextState
        }
        this.applyDirectoryBootstrap(boot)
      }
      catch (e) {
        this.error = e instanceof Error ? e.message : String(e)
      }
      finally {
        this.loading = false
        setRefreshPromise(null)
      }
    })()
    setRefreshPromise(promise)
    return promise
  },
  async checkPluginUpdatesFromRegistry(this: DirectorySliceStore) {
    this.pluginUpdatesCheckLoading = true
    this.error = null
    try {
      const ids = this.catalog.map(c => c.id)
      this.pluginUpdateById = await checkPluginUpdates(ids)
    }
    catch (e) {
      this.error = e instanceof Error ? e.message : String(e)
    }
    finally {
      this.pluginUpdatesCheckLoading = false
    }
  },
  async installPluginFromLocalZip(this: DirectorySliceStore, pluginId: string, zipPath: string) {
    this.extractingPluginId = pluginId
    this.error = null
    try {
      await extractPluginZip(zipPath, pluginId)
      await this.refresh()
      this.bootstrapEpoch += 1
    }
    catch (e) {
      this.error = e instanceof Error ? e.message : String(e)
      throw e
    }
    finally {
      this.extractingPluginId = null
    }
  },
  /** 开发者模式：文件监听触发后刷新 catalog 与 bootstrap（整壳/插槽热重载）。 */
  async onPluginFilesChanged(this: DirectorySliceStore) {
    await this.refresh()
    this.bootstrapEpoch += 1
    await this.syncDirectoryPluginBootstrap()
  },
}

export interface DirectorySliceStore extends InstalledSliceStore {
  loading: boolean
  error: string | null
  catalog: DirectoryPluginCatalogEntry[]
  catalogCandidatesBySlot: Record<string, string[]>
  developerMode: boolean
  bootstrapUiSlots: PluginUiSlotInfo[]
  bootstrapEpoch: number
  pluginUpdateById: Record<string, PluginUpdateInfo>
  pluginUpdatesCheckLoading: boolean
  extractingPluginId: string | null
  applyDirectoryBootstrap(boot: DirectoryPluginBootstrap): void
  syncDirectoryPluginBootstrap(): Promise<void>
}

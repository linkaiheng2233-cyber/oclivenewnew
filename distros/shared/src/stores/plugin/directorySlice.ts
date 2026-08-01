import type {
  DirectoryPluginBootstrap,
  DirectoryPluginCatalogEntry,
  PluginUiSlotInfo,
  PluginUpdateInfo,
} from '@oclive/shared/api'
import type { InstalledSliceStore } from './installedSlice'
import {
  checkPluginUpdates,
  extractPluginZip,
  getDirectoryPluginBootstrap,
  getDirectoryPluginCatalog,
  getPluginState,
} from '@oclive/shared/api'
import { useRoleStore } from '../roleStore'
import {
  catalogEqual,
  clonePluginState,
  getRefreshPromise,
  rolePluginStateEqual,
  setRefreshPromise,
} from './constants'

const bootstrapRequestGenerationByRole = new Map<string, number>()
let activeRefreshCount = 0

function nextBootstrapGeneration(roleId: string): number {
  const next = (bootstrapRequestGenerationByRole.get(roleId) ?? 0) + 1
  bootstrapRequestGenerationByRole.set(roleId, next)
  return next
}

function isCurrentBootstrapGeneration(roleId: string, generation: number): boolean {
  return bootstrapRequestGenerationByRole.get(roleId) === generation
}

export function directoryState() {
  return {
    loading: false,
    error: null as string | null,
    catalog: [] as DirectoryPluginCatalogEntry[],
    /** Precomputed catalog: non-shell plugin ids per slot (sorted) */
    catalogCandidatesBySlot: {} as Record<string, string[]>,
    /** Matches `get_directory_plugin_bootstrap.developer_mode` (extra plugin roots scan, etc.) */
    developerMode: false,
    /** Latest bootstrap embed slot list (matches `get_directory_plugin_bootstrap.uiSlots`) */
    bootstrapUiSlots: [] as PluginUiSlotInfo[],
    /** Embed slot components re-fetch bootstrap after this changes */
    bootstrapEpoch: 0,
    /** Per-plugin slot remount generation (plugin FS hot reload; independent of catalog refresh). */
    slotReloadByPluginId: {} as Record<string, number>,
    /** Latest `check_plugin_updates` result (by plugin id) */
    pluginUpdateById: {} as Record<string, PluginUpdateInfo>,
    pluginUpdatesCheckLoading: false,
    extractingPluginId: null as string | null,
  }
}

export const directoryActions = {
  /** Update developer mode and slot descriptors from bootstrap DTO (shared by slots, `refresh`, `sync`). */
  applyDirectoryBootstrap(this: DirectorySliceStore, boot: DirectoryPluginBootstrap) {
    this.developerMode = boot.developerMode ?? false
    this.bootstrapUiSlots = boot.uiSlots ?? []
  },
  /** After role switch or plugin enable change: update host events and developer mode (no catalog fetch) */
  async syncDirectoryPluginBootstrap(this: DirectorySliceStore) {
    const roleId = useRoleStore().currentRoleId
    const generation = nextBootstrapGeneration(roleId)
    // A bootstrap sync supersedes an older full refresh for the same role.
    setRefreshPromise(roleId, null)
    try {
      const boot = await getDirectoryPluginBootstrap(roleId)
      if (!isCurrentBootstrapGeneration(roleId, generation) || useRoleStore().currentRoleId !== roleId)
        return
      this.applyDirectoryBootstrap(boot)
    }
    catch (e) {
      if (isCurrentBootstrapGeneration(roleId, generation) && useRoleStore().currentRoleId === roleId)
        this.error = e instanceof Error ? e.message : String(e)
    }
  },
  async refresh(this: DirectorySliceStore) {
    const roleId = useRoleStore().currentRoleId
    const existing = getRefreshPromise(roleId)
    if (existing)
      return existing
    const generation = nextBootstrapGeneration(roleId)
    activeRefreshCount += 1
    this.loading = true
    this.error = null
    const promise = (async () => {
      try {
        const [cat, bundle, boot] = await Promise.all([
          getDirectoryPluginCatalog(),
          getPluginState(roleId),
          getDirectoryPluginBootstrap(roleId),
        ])
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
        if (!isCurrentBootstrapGeneration(roleId, generation) || useRoleStore().currentRoleId !== roleId)
          return
        this.pluginStateBundle = {
          role: clonePluginState(bundle.role),
          globalDefaults: clonePluginState(bundle.globalDefaults),
        }
        const st
          = this.persistScope === 'role' ? bundle.role : bundle.globalDefaults
        const nextState = clonePluginState(st)
        if (!rolePluginStateEqual(this.pluginState, nextState)) {
          this.pluginState = nextState
        }
        this.applyDirectoryBootstrap(boot)
      }
      catch (e) {
        if (isCurrentBootstrapGeneration(roleId, generation) && useRoleStore().currentRoleId === roleId)
          this.error = e instanceof Error ? e.message : String(e)
      }
      finally {
        activeRefreshCount = Math.max(0, activeRefreshCount - 1)
        this.loading = activeRefreshCount > 0
      }
    })()
    setRefreshPromise(roleId, promise)
    const clearCurrentRefresh = () => {
      if (getRefreshPromise(roleId) === promise)
        setRefreshPromise(roleId, null)
    }
    void promise.then(clearCurrentRefresh, clearCurrentRefresh)
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
  bumpPluginSlotReload(this: DirectorySliceStore, pluginIds?: readonly string[]) {
    const ids
      = pluginIds && pluginIds.length > 0
        ? pluginIds
        : (this.bootstrapUiSlots ?? []).map(s => s.pluginId)
    if (ids.length === 0)
      return
    const next = { ...this.slotReloadByPluginId }
    for (const id of new Set(ids))
      next[id] = (next[id] ?? 0) + 1
    this.slotReloadByPluginId = next
    this.bootstrapEpoch += 1
  },
  /** Developer mode: refresh catalog and bootstrap after file watcher fires (full-shell / slot hot reload) */
  async onPluginFilesChanged(this: DirectorySliceStore) {
    this.bumpPluginSlotReload()
    await this.refresh()
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
  slotReloadByPluginId: Record<string, number>
  bumpPluginSlotReload: (pluginIds?: readonly string[]) => void
  pluginUpdateById: Record<string, PluginUpdateInfo>
  pluginUpdatesCheckLoading: boolean
  extractingPluginId: string | null
  applyDirectoryBootstrap: (boot: DirectoryPluginBootstrap) => void
  syncDirectoryPluginBootstrap: () => Promise<void>
}

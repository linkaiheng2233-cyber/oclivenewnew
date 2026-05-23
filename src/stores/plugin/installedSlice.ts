import type { DirectoryPluginCatalogEntry, RolePluginState } from '../../api'
import {
  resetPluginStateToRoleDefault,
  saveGlobalPluginState,
  savePluginState,
} from '../../api'
import { rt } from '../../i18n/runtimeT'
import { useRoleStore } from '../roleStore'
import {
  buildSlotOrderSignature,
  clonePluginState,
  emptyState,
  SLOT_CHAT_TOOLBAR,
  type PluginPersistScope,
  type SlotOrderMemo,
} from './constants'

export function installedState() {
  return {
    pluginState: emptyState() as RolePluginState,
    /** 最近一次 `get_plugin_state` 的按角色 / 全局原始行（切换「保存到」时回填）。 */
    pluginStateBundle: null as {
      role: RolePluginState
      globalDefaults: RolePluginState
    } | null,
    /** 保存目标：`role` 写入当前角色；`global` 写入跨角色默认。 */
    persistScope: 'role' as PluginPersistScope,
    slotOrderMemoBySlot: {} as Record<string, SlotOrderMemo>,
  }
}

export const installedActions = {
  /** 下拉布局：设置某 UI 插槽的插件顺序（不含已禁用贡献的过滤，由 UI 传入有效 id 列表）。 */
  setSlotPluginIds(this: InstalledSliceStore, slot: string, orderedIds: string[]) {
    const candidates = this.catalogCandidatesBySlot[slot] ?? []
    const candidateSet = new Set(candidates)
    const seen = new Set<string>()
    const out: string[] = []
    for (const id of orderedIds) {
      if (candidateSet.has(id) && !seen.has(id)) {
        out.push(id)
        seen.add(id)
      }
    }
    for (const id of candidates) {
      if (!seen.has(id)) {
        out.push(id)
      }
    }
    this.pluginState.slot_order = {
      ...this.pluginState.slot_order,
      [slot]: out,
    }
    delete this.slotOrderMemoBySlot[slot]
  },
  setPersistScope(this: InstalledSliceStore, scope: PluginPersistScope) {
    if (this.persistScope === scope) {
      return
    }
    const b = this.pluginStateBundle
    if (b) {
      this.pluginState
        = scope === 'role'
          ? clonePluginState(b.role)
          : clonePluginState(b.globalDefaults)
    }
    this.persistScope = scope
  },
  async persist(this: InstalledSliceStore) {
    const roleId = useRoleStore().currentRoleId
    if (this.persistScope === 'global') {
      await saveGlobalPluginState(this.pluginState)
      if (this.pluginStateBundle) {
        this.pluginStateBundle.globalDefaults = clonePluginState(
          this.pluginState,
        )
      }
    }
    else {
      await savePluginState(roleId, this.pluginState)
      if (this.pluginStateBundle) {
        this.pluginStateBundle.role = clonePluginState(this.pluginState)
      }
    }
    this.bootstrapEpoch += 1
    await this.syncDirectoryPluginBootstrap()
  },
  async resetToRolePackDefault(this: InstalledSliceStore) {
    const roleId = useRoleStore().currentRoleId
    await resetPluginStateToRoleDefault(roleId)
    await this.refresh()
    this.bootstrapEpoch += 1
  },
  isPluginDisabled(this: InstalledSliceStore, id: string): boolean {
    return this.pluginState.disabled_plugins.includes(id)
  },
  setPluginDisabled(this: InstalledSliceStore, id: string, disabled: boolean) {
    if (!disabled) {
      const entry = this.catalog.find(c => c.id === id)
      if (entry && entry.dependencyStatus !== 'ok') {
        throw new Error(
          rt('pluginWorkbench.errors.dependencyNotMet', {
            id,
            issues: (entry.dependencyIssues ?? []).join('; '),
          }),
        )
      }
    }
    const set = new Set(this.pluginState.disabled_plugins)
    if (disabled) {
      set.add(id)
    }
    else {
      set.delete(id)
    }
    this.pluginState.disabled_plugins = [...set].sort()
  },
  batchDisablePluginIds(this: InstalledSliceStore, ids: string[]) {
    const set = new Set(this.pluginState.disabled_plugins)
    for (const id of ids) {
      set.add(id)
    }
    this.pluginState.disabled_plugins = [...set].sort()
  },
  batchEnablePluginIds(this: InstalledSliceStore, ids: string[]) {
    for (const id of ids) {
      const entry = this.catalog.find(c => c.id === id)
      if (entry && entry.dependencyStatus !== 'ok') {
        throw new Error(
          rt('pluginWorkbench.errors.dependencyNotMet', {
            id,
            issues: (entry.dependencyIssues ?? []).join('; '),
          }),
        )
      }
    }
    const set = new Set(this.pluginState.disabled_plugins)
    for (const id of ids) {
      set.delete(id)
    }
    this.pluginState.disabled_plugins = [...set].sort()
  },
  /** 对「检测到有更新」的插件提示需 zip 导入（在线静默更新未接入）。 */
  async batchUpdatePluginIds(
    this: InstalledSliceStore,
    ids: string[],
  ): Promise<{ count: number, targets: string[] }> {
    await this.checkPluginUpdatesFromRegistry()
    const targets = ids.filter(id => this.pluginUpdateById[id]?.hasUpdate)
    if (targets.length === 0) {
      return { count: 0, targets: [] }
    }
    return { count: targets.length, targets }
  },
  /** 某插槽下、按 manifest 声明了该槽的非整壳插件 id 顺序（含未在 slot_order 中的，字典序补全）。 */
  pluginsOrderedForSlot(this: InstalledSliceStore, slot: string): string[] {
    const candidates = this.catalogCandidatesBySlot[slot] ?? []
    const order = this.pluginState.slot_order[slot] ?? []
    const signature = buildSlotOrderSignature(candidates, order)
    const memo = this.slotOrderMemoBySlot[slot]
    if (memo && memo.signature === signature) {
      return [...memo.value]
    }
    const candidateSet = new Set(candidates)
    const seen = new Set<string>()
    const out: string[] = []
    for (const id of order) {
      if (candidateSet.has(id) && !seen.has(id)) {
        out.push(id)
        seen.add(id)
      }
    }
    for (const id of candidates) {
      if (!seen.has(id)) {
        out.push(id)
      }
    }
    this.slotOrderMemoBySlot[slot] = { signature, value: out }
    return out
  },
  isSlotContributionDisabled(this: InstalledSliceStore, slot: string, pluginId: string): boolean {
    const list = this.pluginState.disabled_slot_contributions[slot] ?? []
    return list.includes(pluginId)
  },
  setSlotContributionDisabled(
    this: InstalledSliceStore,
    slot: string,
    pluginId: string,
    disabled: boolean,
  ) {
    const cur = [...(this.pluginState.disabled_slot_contributions[slot] ?? [])]
    const i = cur.indexOf(pluginId)
    if (disabled && i < 0) {
      cur.push(pluginId)
    }
    else if (!disabled && i >= 0) {
      cur.splice(i, 1)
    }
    this.pluginState.disabled_slot_contributions = {
      ...this.pluginState.disabled_slot_contributions,
      [slot]: cur,
    }
  },
  /** 设置某插件在某槽的选中外观（`appearance_id`）；空字符串表示清除为 manifest 默认。 */
  setSlotAppearance(
    this: InstalledSliceStore,
    pluginId: string,
    slot: string,
    appearanceId: string,
  ) {
    const pid = pluginId.trim()
    const sl = slot.trim()
    if (!pid || !sl)
      return
    const nextOuter: Record<string, Record<string, string>> = {
      ...(this.pluginState.slot_appearance ?? {}),
    }
    const inner = { ...(nextOuter[pid] ?? {}) }
    const aid = appearanceId.trim()
    if (aid === '') {
      delete inner[sl]
    }
    else {
      inner[sl] = aid
    }
    if (Object.keys(inner).length === 0) {
      delete nextOuter[pid]
    }
    else {
      nextOuter[pid] = inner
    }
    this.pluginState = {
      ...this.pluginState,
      slot_appearance: nextOuter,
    }
  },
  movePluginInSlotOrder(this: InstalledSliceStore, slot: string, fromIndex: number, toIndex: number) {
    const ids = [...this.pluginsOrderedForSlot(slot)]
    if (
      fromIndex < 0
      || toIndex < 0
      || fromIndex >= ids.length
      || toIndex >= ids.length
    ) {
      return
    }
    const [m] = ids.splice(fromIndex, 1)
    if (m === undefined) {
      return
    }
    ids.splice(toIndex, 0, m)
    this.pluginState.slot_order = {
      ...this.pluginState.slot_order,
      [slot]: ids,
    }
  },
  // --- 兼容旧名（chat_toolbar）---
  toolbarPluginsOrdered(this: InstalledSliceStore): string[] {
    return this.pluginsOrderedForSlot(SLOT_CHAT_TOOLBAR)
  },
  moveToolbarPlugin(this: InstalledSliceStore, fromIndex: number, toIndex: number) {
    this.movePluginInSlotOrder(SLOT_CHAT_TOOLBAR, fromIndex, toIndex)
  },
  isToolbarContributionDisabled(this: InstalledSliceStore, pluginId: string): boolean {
    return this.isSlotContributionDisabled(SLOT_CHAT_TOOLBAR, pluginId)
  },
  setToolbarContributionDisabled(this: InstalledSliceStore, pluginId: string, disabled: boolean) {
    this.setSlotContributionDisabled(SLOT_CHAT_TOOLBAR, pluginId, disabled)
  },
}

export interface InstalledSliceStore {
  pluginState: RolePluginState
  pluginStateBundle: {
    role: RolePluginState
    globalDefaults: RolePluginState
  } | null
  persistScope: PluginPersistScope
  slotOrderMemoBySlot: Record<string, SlotOrderMemo>
  catalog: DirectoryPluginCatalogEntry[]
  catalogCandidatesBySlot: Record<string, string[]>
  pluginUpdateById: Record<string, { hasUpdate?: boolean }>
  bootstrapEpoch: number
  refresh(): Promise<void>
  syncDirectoryPluginBootstrap(): Promise<void>
  checkPluginUpdatesFromRegistry(): Promise<void>
  pluginsOrderedForSlot(slot: string): string[]
}

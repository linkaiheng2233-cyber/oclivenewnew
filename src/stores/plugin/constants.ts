import type { DirectoryPluginCatalogEntry, RolePluginState, UiSlotVariantInfo } from '../../api'

export interface SlotOrderMemo {
  signature: string
  value: string[]
}

/** 并发 `refresh()` 合并为单次执行（共享 Promise）。 */
export let refreshPromise: Promise<void> | null = null

export function setRefreshPromise(p: Promise<void> | null): void {
  refreshPromise = p
}

/** 聊天输入区上方工具栏 */
export const SLOT_CHAT_TOOLBAR = 'chat_toolbar'
/** 应用内「设置 → 插件扩展」页嵌入 */
export const SLOT_SETTINGS_PANEL = 'settings.panel'
/** 左侧角色详情区底部（立绘与名称下方） */
export const SLOT_ROLE_DETAIL = 'role.detail'
/** 左侧栏角色块下方（好感度条上方），整列侧栏扩展区 */
export const SLOT_SIDEBAR = 'sidebar'
/** 右侧聊天列顶部（消息列表上方） */
export const SLOT_CHAT_HEADER = 'chat.header'
/** 插件管理面板内嵌 */
export const SLOT_SETTINGS_PLUGINS = 'settings.plugins'
/** 设置对话框 · 扩展区（常规） */
export const SLOT_SETTINGS_ADVANCED = 'settings.advanced'
/** 全局浮层模板区 */
export const SLOT_OVERLAY_FLOATING = 'overlay.floating'
/** 快捷键说明 / 启动器聚合 */
export const SLOT_LAUNCHER_PALETTE = 'launcher.palette'
/** 调试面板扩展 */
export const SLOT_DEBUG_DOCK = 'debug.dock'

/** 与后端 `EMBEDDED_UI_SLOT_NAMES` 顺序一致（用于遍历）。 */
export const ALL_EMBEDDED_SLOT_NAMES: readonly string[] = [
  SLOT_CHAT_TOOLBAR,
  SLOT_SETTINGS_PANEL,
  SLOT_ROLE_DETAIL,
  SLOT_SIDEBAR,
  SLOT_CHAT_HEADER,
  SLOT_SETTINGS_PLUGINS,
  SLOT_SETTINGS_ADVANCED,
  SLOT_OVERLAY_FLOATING,
  SLOT_LAUNCHER_PALETTE,
  SLOT_DEBUG_DOCK,
]

export function emptyState(): RolePluginState {
  return {
    shellPluginId: '',
    disabled_plugins: [],
    slot_order: {},
    disabled_slot_contributions: {},
    slot_appearance: {},
    force_iframe_mode: false,
  }
}

export function arraysEqual(a: string[] = [], b: string[] = []): boolean {
  if (a === b)
    return true
  if (a.length !== b.length)
    return false
  for (let i = 0; i < a.length; i += 1) {
    if (a[i] !== b[i])
      return false
  }
  return true
}

export function recordOfArraysEqual(
  a: Record<string, string[]> = {},
  b: Record<string, string[]> = {},
): boolean {
  const ka = Object.keys(a)
  const kb = Object.keys(b)
  if (ka.length !== kb.length)
    return false
  for (const k of ka) {
    if (!(k in b))
      return false
    if (!arraysEqual(a[k], b[k]))
      return false
  }
  return true
}

function uiSlotVariantsEqual(
  a: UiSlotVariantInfo[] | undefined,
  b: UiSlotVariantInfo[] | undefined,
): boolean {
  const x = a ?? []
  const y = b ?? []
  if (x.length !== y.length)
    return false
  for (let i = 0; i < x.length; i += 1) {
    const p = x[i]
    const q = y[i]
    if (
      p.slot !== q.slot
      || p.appearanceId !== q.appearanceId
      || (p.label ?? '') !== (q.label ?? '')
    ) {
      return false
    }
  }
  return true
}

export function catalogEqual(
  a: DirectoryPluginCatalogEntry[],
  b: DirectoryPluginCatalogEntry[],
): boolean {
  if (a === b)
    return true
  if (a.length !== b.length)
    return false
  for (let i = 0; i < a.length; i += 1) {
    const x = a[i]
    const y = b[i]
    if (
      x.id !== y.id
      || x.version !== y.version
      || (x.pluginType ?? null) !== (y.pluginType ?? null)
      || (x.hasUiSettings ?? false) !== (y.hasUiSettings ?? false)
      || x.hasRpcProcess !== y.hasRpcProcess
      || (x.declaresRpcMethods ?? false) !== (y.declaresRpcMethods ?? false)
      || x.isShell !== y.isShell
      || (x.dependencyStatus ?? 'ok') !== (y.dependencyStatus ?? 'ok')
      || !arraysEqual(x.uiSlotNames ?? [], y.uiSlotNames ?? [])
      || !uiSlotVariantsEqual(x.uiSlotVariants, y.uiSlotVariants)
      || !arraysEqual(x.provides ?? [], y.provides ?? [])
      || !arraysEqual(x.dependencyIssues ?? [], y.dependencyIssues ?? [])
    ) {
      return false
    }
  }
  return true
}

function slotAppearanceEqual(
  a: Record<string, Record<string, string>> | undefined,
  b: Record<string, Record<string, string>> | undefined,
): boolean {
  const aa = a ?? {}
  const bb = b ?? {}
  const keysA = Object.keys(aa).sort()
  const keysB = Object.keys(bb).sort()
  if (keysA.length !== keysB.length)
    return false
  for (let i = 0; i < keysA.length; i += 1) {
    if (keysA[i] !== keysB[i])
      return false
    const pid = keysA[i]!
    const ia = aa[pid]!
    const ib = bb[pid]!
    const skA = Object.keys(ia).sort()
    const skB = Object.keys(ib).sort()
    if (skA.length !== skB.length)
      return false
    for (let j = 0; j < skA.length; j += 1) {
      if (skA[j] !== skB[j])
        return false
      const slot = skA[j]!
      if (ia[slot] !== ib[slot])
        return false
    }
  }
  return true
}

export function rolePluginStateEqual(a: RolePluginState, b: RolePluginState): boolean {
  return (
    a.shellPluginId === b.shellPluginId
    && (a.force_iframe_mode ?? false) === (b.force_iframe_mode ?? false)
    && arraysEqual(a.disabled_plugins ?? [], b.disabled_plugins ?? [])
    && recordOfArraysEqual(a.slot_order ?? {}, b.slot_order ?? {})
    && recordOfArraysEqual(
      a.disabled_slot_contributions ?? {},
      b.disabled_slot_contributions ?? {},
    )
    && slotAppearanceEqual(a.slot_appearance, b.slot_appearance)
  )
}

export function buildSlotOrderSignature(candidates: string[], order: string[]): string {
  return `${candidates.join('\u001F')}\u001E${order.join('\u001F')}`
}

export type PluginPersistScope = 'role' | 'global'

export function clonePluginState(s: RolePluginState): RolePluginState {
  const sa = s.slot_appearance ?? {}
  const slot_appearance: Record<string, Record<string, string>> = {}
  for (const pid of Object.keys(sa)) {
    slot_appearance[pid] = { ...sa[pid] }
  }
  return {
    shellPluginId: s.shellPluginId ?? '',
    disabled_plugins: [...(s.disabled_plugins ?? [])],
    slot_order: { ...s.slot_order },
    disabled_slot_contributions: { ...s.disabled_slot_contributions },
    slot_appearance,
    force_iframe_mode: s.force_iframe_mode ?? false,
  }
}

export type PluginPanelMainTab = 'graph' | 'layout'

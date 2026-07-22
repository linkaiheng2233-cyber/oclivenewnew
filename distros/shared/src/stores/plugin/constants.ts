import type { DirectoryPluginCatalogEntry, RolePluginState, UiSlotVariantInfo } from '@oclive/shared/api'

export interface SlotOrderMemo {
  signature: string
  value: string[]
}

/** Coalesce concurrent `refresh()` calls only within the same role dimension. */
const refreshPromisesByRole = new Map<string, Promise<void>>()

export function pluginRefreshKey(roleId: string): string {
  const normalized = roleId.trim()
  return normalized || '__default__'
}

export function getRefreshPromise(roleId: string): Promise<void> | undefined {
  return refreshPromisesByRole.get(pluginRefreshKey(roleId))
}

export function setRefreshPromise(roleId: string, promise: Promise<void> | null): void {
  const key = pluginRefreshKey(roleId)
  if (promise)
    refreshPromisesByRole.set(key, promise)
  else
    refreshPromisesByRole.delete(key)
}

/** Side-channel plugins that stay available in `pure_chat` (not gated by story mode). */
export const PURE_CHAT_PLATFORM_PLUGIN_IDS: readonly string[] = [
  'com.oclive.voice.asr',
]

export function isPureChatPlatformPlugin(pluginId: string): boolean {
  return PURE_CHAT_PLATFORM_PLUGIN_IDS.includes(pluginId)
}

/** Toolbar above chat input */
export const SLOT_CHAT_TOOLBAR = 'chat_toolbar'
/** Embed on in-app Settings → Plugin extensions page */
export const SLOT_SETTINGS_PANEL = 'settings.panel'
/** Bottom of left role detail (below portrait and name) */
export const SLOT_ROLE_DETAIL = 'role.detail'
/** Below left sidebar role block (above favorability bar); full sidebar extension area */
export const SLOT_SIDEBAR = 'sidebar'
/** Top of right chat column (above message list) */
export const SLOT_CHAT_HEADER = 'chat.header'
/** Embed in plugin manager panel */
export const SLOT_SETTINGS_PLUGINS = 'settings.plugins'
/** Settings dialog · extensions area (regular) */
export const SLOT_SETTINGS_ADVANCED = 'settings.advanced'
/** Global overlay template area */
export const SLOT_OVERLAY_FLOATING = 'overlay.floating'
/** Shortcut help / launcher palette */
export const SLOT_LAUNCHER_PALETTE = 'launcher.palette'
/** Debug panel extension */
export const SLOT_DEBUG_DOCK = 'debug.dock'

/** Same order as backend `EMBEDDED_UI_SLOT_NAMES` (for iteration). */
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

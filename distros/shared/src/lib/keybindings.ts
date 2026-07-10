export type KeybindingScope = 'app' | 'hold' | 'global'

export type KeybindingActionId =
  | 'app.openSettings'
  | 'app.openPluginManager'
  | 'app.openModelManager'
  | 'app.toggleDebug'
  | 'app.openShortcutHelp'
  | 'voice.holdToTalk'
  | 'plugin.openLauncher'
  | 'plugin.openSlot'

export interface KeybindingAction {
  id: KeybindingActionId
  titleKey: string
  /** Default display string, e.g. "Ctrl+Shift+S" */
  defaultBinding: string
  scope: KeybindingScope
  /** When true, the action is only available in immersive roles. */
  immersiveOnly?: boolean
}

export interface UnifiedBinding {
  id: string
  actionId: KeybindingActionId
  accelerator: string
  enabled: boolean
  /**
   * Optional params for actions that need configuration.
   * Keep values JSON-serializable.
   */
  params?: Record<string, unknown>
}

export interface UnifiedBindingsFileV1 {
  schemaVersion: 1
  bindings: UnifiedBinding[]
}

const STORAGE_KEY = 'oclive.keybindings.unified.v1'

export const KEYBINDING_ACTIONS: KeybindingAction[] = [
  { id: 'app.openSettings', titleKey: 'keybindings.actions.openSettings', defaultBinding: 'Ctrl+Shift+S', scope: 'app' },
  { id: 'app.openPluginManager', titleKey: 'keybindings.actions.openPluginManager', defaultBinding: 'Ctrl+Shift+F', scope: 'app', immersiveOnly: true },
  { id: 'app.openModelManager', titleKey: 'keybindings.actions.openModelManager', defaultBinding: 'Ctrl+Shift+M', scope: 'app' },
  { id: 'app.toggleDebug', titleKey: 'keybindings.actions.toggleDebug', defaultBinding: 'Ctrl+Shift+D', scope: 'app', immersiveOnly: true },
  { id: 'app.openShortcutHelp', titleKey: 'keybindings.actions.openShortcutHelp', defaultBinding: 'Ctrl+LongPress', scope: 'app', immersiveOnly: false },
  { id: 'voice.holdToTalk', titleKey: 'keybindings.actions.holdToTalk', defaultBinding: 'V', scope: 'hold' },
  { id: 'plugin.openLauncher', titleKey: 'keybindings.actions.openPluginLauncher', defaultBinding: '', scope: 'global' },
  { id: 'plugin.openSlot', titleKey: 'keybindings.actions.openPluginSlot', defaultBinding: '', scope: 'global' },
]

export function getKeybindingAction(id: KeybindingActionId): KeybindingAction | undefined {
  return KEYBINDING_ACTIONS.find(a => a.id === id)
}

function newId(prefix: string): string {
  if (typeof crypto !== 'undefined' && 'randomUUID' in crypto && typeof crypto.randomUUID === 'function')
    return `${prefix}-${crypto.randomUUID()}`
  return `${prefix}-${Date.now()}-${Math.random().toString(16).slice(2)}`
}

export function createDefaultUnifiedBindingsFile(): UnifiedBindingsFileV1 {
  const bindings: UnifiedBinding[] = KEYBINDING_ACTIONS
    .filter(a => a.defaultBinding !== '')
    .map(a => ({
      id: newId('kb'),
      actionId: a.id,
      accelerator: a.defaultBinding,
      enabled: true,
    }))

  return { schemaVersion: 1, bindings }
}

export function loadUnifiedBindingsFile(): UnifiedBindingsFileV1 {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw)
      return createDefaultUnifiedBindingsFile()
    const parsed = JSON.parse(raw) as Partial<UnifiedBindingsFileV1>
    if (parsed && parsed.schemaVersion === 1 && Array.isArray(parsed.bindings)) {
      const cleaned = parsed.bindings
        .filter(b => b && typeof b === 'object')
        .map((b) => {
          const row = b as Partial<UnifiedBinding>
          return {
            id: typeof row.id === 'string' && row.id ? row.id : newId('kb'),
            actionId: (row.actionId ?? '') as KeybindingActionId,
            accelerator: typeof row.accelerator === 'string' ? row.accelerator : '',
            enabled: typeof row.enabled === 'boolean' ? row.enabled : true,
            params: typeof row.params === 'object' && row.params ? row.params as Record<string, unknown> : undefined,
          } satisfies UnifiedBinding
        })
        .filter(b => KEYBINDING_ACTIONS.some(a => a.id === b.actionId))
      return { schemaVersion: 1, bindings: cleaned }
    }
  }
  catch {
    // ignore
  }
  return createDefaultUnifiedBindingsFile()
}

export function saveUnifiedBindingsFile(file: UnifiedBindingsFileV1): void {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(file))
}

export function normalizeBindingDisplay(binding: string): string {
  return (binding ?? '').trim().replace(/\s+/g, '')
}

export function describeBinding(binding: string): string {
  const b = normalizeBindingDisplay(binding)
  if (!b)
    return ''
  if (b === 'Ctrl+LongPress')
    return 'Ctrl (hold)'
  return b.replaceAll('+', ' + ')
}

export function describeBindingOrUnbound(binding: string): string {
  const d = describeBinding(binding)
  return d || '—'
}

function keyForEvent(e: KeyboardEvent): string {
  // Prefer code for letters to avoid layout issues; keep non-letters by key.
  if (e.code && /^Key[A-Z]$/.test(e.code))
    return e.code.slice(3)
  const k = (e.key || '').toUpperCase()
  if (k === ' ')
    return 'Space'
  if (k === 'ESCAPE')
    return 'Esc'
  return k
}

export function eventToBinding(e: KeyboardEvent): string {
  const parts: string[] = []
  if (e.ctrlKey)
    parts.push('Ctrl')
  if (e.shiftKey)
    parts.push('Shift')
  if (e.altKey)
    parts.push('Alt')
  if (e.metaKey)
    parts.push('Meta')
  const k = keyForEvent(e)
  if (k && k !== 'CTRL' && k !== 'SHIFT' && k !== 'ALT' && k !== 'META')
    parts.push(k)
  return parts.join('+')
}

export function bindingMatchesEvent(binding: string, e: KeyboardEvent): boolean {
  const b = normalizeBindingDisplay(binding)
  if (!b)
    return false
  if (b === 'Ctrl+LongPress')
    return false
  return normalizeBindingDisplay(eventToBinding(e)) === b
}

export function shouldIgnoreTarget(target: EventTarget | null | undefined): boolean {
  const el = target instanceof Element ? target : null
  if (!el)
    return false
  if (el.closest('[contenteditable="true"]'))
    return true
  const tag = el.tagName.toLowerCase()
  if (tag === 'input' || tag === 'textarea' || tag === 'select')
    return true
  if (el.getAttribute('role') === 'textbox')
    return true
  return false
}

export function actionScope(actionId: KeybindingActionId): KeybindingScope {
  return getKeybindingAction(actionId)?.scope ?? 'app'
}

export function isHoldAction(actionId: KeybindingActionId): boolean {
  return actionScope(actionId) === 'hold'
}

export function normalizeAccelerator(accel: string): string {
  return normalizeBindingDisplay(accel)
}

export function getDefaultBindingForAction(actionId: KeybindingActionId): string {
  return normalizeAccelerator(getKeybindingAction(actionId)?.defaultBinding ?? '')
}

export function getEffectiveAcceleratorForRow(row: UnifiedBinding): string {
  const custom = normalizeAccelerator(row.accelerator)
  if (custom)
    return custom
  return getDefaultBindingForAction(row.actionId)
}

export function getEffectiveBindingsByActionId(file: UnifiedBindingsFileV1): Map<KeybindingActionId, UnifiedBinding[]> {
  const out = new Map<KeybindingActionId, UnifiedBinding[]>()
  for (const b of file.bindings) {
    const arr = out.get(b.actionId) ?? []
    arr.push(b)
    out.set(b.actionId, arr)
  }
  return out
}

export function getPrimaryEffectiveAcceleratorForAction(
  file: UnifiedBindingsFileV1,
  actionId: KeybindingActionId,
): string {
  const candidates = file.bindings.filter(b => b.actionId === actionId && b.enabled)
  if (candidates.length === 0)
    return getDefaultBindingForAction(actionId)
  const first = candidates[0]!
  return getEffectiveAcceleratorForRow(first)
}

export interface BindingConflict {
  accelerator: string
  scope: KeybindingScope
  bindingIds: string[]
  actionIds: KeybindingActionId[]
}

export function detectBindingConflicts(file: UnifiedBindingsFileV1): BindingConflict[] {
  const bucket = new Map<string, { scope: KeybindingScope, ids: string[], actions: KeybindingActionId[] }>()
  for (const b of file.bindings) {
    if (!b.enabled)
      continue
    const accel = getEffectiveAcceleratorForRow(b)
    if (!accel)
      continue
    const scope = actionScope(b.actionId)
    // Ctrl+LongPress is a special hint, allow coexistence with other Ctrl combos
    if (accel === 'Ctrl+LongPress')
      continue
    const key = `${scope}:${accel}`
    const cur = bucket.get(key) ?? { scope, ids: [], actions: [] }
    cur.ids.push(b.id)
    cur.actions.push(b.actionId)
    bucket.set(key, cur)
  }
  const conflicts: BindingConflict[] = []
  for (const [k, v] of bucket) {
    if (v.ids.length > 1) {
      const accel = k.split(':').slice(1).join(':')
      conflicts.push({ accelerator: accel, scope: v.scope, bindingIds: v.ids, actionIds: v.actions })
    }
  }
  return conflicts
}

export function isValidHoldAccelerator(accel: string): boolean {
  const a = normalizeAccelerator(accel)
  if (!a)
    return false
  // For hold-to-talk we only support single key (no modifiers) to avoid accidental grabs.
  return !a.includes('+') && a !== 'Ctrl' && a !== 'Shift' && a !== 'Alt' && a !== 'Meta'
}


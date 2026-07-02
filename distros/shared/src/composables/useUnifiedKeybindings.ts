import type { ComputedRef, Ref } from 'vue'
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import {
  actionScope,
  bindingMatchesEvent,
  detectBindingConflicts,
  getPrimaryEffectiveAcceleratorForAction,
  isValidHoldAccelerator,
  loadUnifiedBindingsFile,
  normalizeAccelerator,
  saveUnifiedBindingsFile,
  shouldIgnoreTarget,
  type KeybindingActionId,
  type UnifiedBinding,
  type UnifiedBindingsFileV1,
} from '@oclive/shared/lib/keybindings'

export interface KeybindingRuntimeAction {
  actionId: Extract<KeybindingActionId, `app.${string}`>
  /**
   * Whether this action is currently available (e.g. immersiveOnly).
   * When false, the binding will not fire.
   */
  enabled: ComputedRef<boolean>
  run: () => void
}

export interface HoldRuntimeAction {
  actionId: Extract<KeybindingActionId, `voice.${string}`>
  enabled: ComputedRef<boolean>
  onStart: () => void
  onStop: () => void
}

export function useUnifiedKeybindings(options: {
  appActions: KeybindingRuntimeAction[]
  holdActions?: HoldRuntimeAction[]
  /**
   * Optional: allow hotkeys even when focus is inside inputs.
   * Default false.
   */
  allowInInputs?: Ref<boolean> | ComputedRef<boolean>
}) {
  const file = ref<UnifiedBindingsFileV1>(loadUnifiedBindingsFile())

  function saveToStorage(): void {
    saveUnifiedBindingsFile(file.value)
  }

  function setFile(next: UnifiedBindingsFileV1): void {
    file.value = next
    saveToStorage()
  }

  function setRow(rowId: string, patch: Partial<UnifiedBinding>): void {
    const next = file.value.bindings.map((b) => {
      if (b.id !== rowId)
        return b
      return { ...b, ...patch }
    })
    setFile({ schemaVersion: 1, bindings: next })
  }

  const conflicts = computed(() => detectBindingConflicts(file.value))

  function appRowsForAction(actionId: KeybindingActionId): UnifiedBinding[] {
    return file.value.bindings.filter(b => b.actionId === actionId && actionScope(b.actionId) === 'app')
  }

  function holdRowsForAction(actionId: KeybindingActionId): UnifiedBinding[] {
    return file.value.bindings.filter(b => b.actionId === actionId && actionScope(b.actionId) === 'hold')
  }

  const pressedHold = new Set<KeybindingActionId>()

  function canProcessEvent(e: KeyboardEvent): boolean {
    if (e.repeat)
      return false
    if (!document.hasFocus() || document.visibilityState !== 'visible')
      return false
    const allow = options.allowInInputs?.value ?? false
    if (!allow && shouldIgnoreTarget(e.target))
      return false
    return true
  }

  function onKeydown(e: KeyboardEvent): void {
    if (!canProcessEvent(e))
      return

    for (const r of options.appActions) {
      if (!r.enabled.value)
        continue
      const accel = getPrimaryEffectiveAcceleratorForAction(file.value, r.actionId)
      if (!accel)
        continue
      if (bindingMatchesEvent(accel, e)) {
        e.preventDefault()
        r.run()
        return
      }
    }

    for (const h of options.holdActions ?? []) {
      if (!h.enabled.value)
        continue
      const rows = holdRowsForAction(h.actionId)
      if (rows.length === 0)
        continue
      const accel = getPrimaryEffectiveAcceleratorForAction(file.value, h.actionId)
      if (!accel || !isValidHoldAccelerator(accel))
        continue
      const key = normalizeAccelerator(e.code && /^Key[A-Z]$/.test(e.code) ? e.code.slice(3) : (e.key || ''))
      if (normalizeAccelerator(accel) !== normalizeAccelerator(key))
        continue
      if (pressedHold.has(h.actionId))
        continue
      pressedHold.add(h.actionId)
      e.preventDefault()
      h.onStart()
      return
    }
  }

  function onKeyup(e: KeyboardEvent): void {
    if (!document.hasFocus() || document.visibilityState !== 'visible')
      return
    const allow = options.allowInInputs?.value ?? false
    if (!allow && shouldIgnoreTarget(e.target))
      return

    for (const h of options.holdActions ?? []) {
      if (!h.enabled.value)
        continue
      if (!pressedHold.has(h.actionId))
        continue
      const accel = getPrimaryEffectiveAcceleratorForAction(file.value, h.actionId)
      if (!accel || !isValidHoldAccelerator(accel))
        continue
      const key = normalizeAccelerator(e.code && /^Key[A-Z]$/.test(e.code) ? e.code.slice(3) : (e.key || ''))
      if (normalizeAccelerator(accel) !== normalizeAccelerator(key))
        continue
      pressedHold.delete(h.actionId)
      e.preventDefault()
      h.onStop()
      return
    }
  }

  onMounted(() => {
    window.addEventListener('keydown', onKeydown)
    window.addEventListener('keyup', onKeyup)
  })

  onBeforeUnmount(() => {
    window.removeEventListener('keydown', onKeydown)
    window.removeEventListener('keyup', onKeyup)
    pressedHold.clear()
  })

  // Keep file fresh if other UI saves it in-place
  watch(
    () => file.value,
    () => {
      // noop: watcher ensures reactivity consumers update
    },
  )

  const effectiveAcceleratorByAction = computed(() => {
    const m = new Map<KeybindingActionId, string>()
    const actionIds = new Set<KeybindingActionId>(file.value.bindings.map(b => b.actionId))
    for (const id of actionIds)
      m.set(id, getPrimaryEffectiveAcceleratorForAction(file.value, id))
    return m
  })

  return {
    file,
    conflicts,
    effectiveAcceleratorByAction,
    saveToStorage,
    setFile,
    setRow,
    appRowsForAction,
    holdRowsForAction,
  }
}


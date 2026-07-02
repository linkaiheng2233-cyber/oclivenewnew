import type { Ref } from 'vue'
import { computed, type ComputedRef, onBeforeUnmount, onMounted, ref } from 'vue'
import { hostEventBus } from '@oclive/shared/lib/hostEventBus'
import { resolveOcliveShell } from './useOcliveShell'
import { getPrimaryEffectiveAcceleratorForAction, loadUnifiedBindingsFile } from '@oclive/shared/lib/keybindings'
import { type HoldRuntimeAction, useUnifiedKeybindings } from '@oclive/shared/composables/useUnifiedKeybindings'

export interface UseGlobalHotkeysOptions {
  simplePluginManagerOpen: Ref<boolean>
  settingsViewOpen: Ref<boolean>
  topMoreOpen: Ref<boolean>
  marketPanelVisible: Ref<boolean> | ComputedRef<boolean>
  modelManagerOpen: Ref<boolean>
  debugVisible: Ref<boolean> | ComputedRef<boolean>
  pluginUiEnabled: Ref<boolean> | ComputedRef<boolean>
  debugUiEnabled?: Ref<boolean> | ComputedRef<boolean>
  openPluginManagerPanel: () => void
  openModelManager: () => void
  toggleDebug: () => void
  closeMarketPanel: () => void
  closeModelManager: () => void
  holdActions?: HoldRuntimeAction[]
}

export function useGlobalHotkeys(opts: UseGlobalHotkeysOptions) {
  const shortcutHelpOpen = ref(false)
  let ctrlLongPressTimer: ReturnType<typeof setTimeout> | null = null
  let ctrlHoldBlockedByPointer = false

  const isTheater = computed(() => resolveOcliveShell() === 'theater')

  function openShortcutHelp(): void {
    shortcutHelpOpen.value = true
    opts.topMoreOpen.value = false
  }

  function openSettingsView(): void {
    opts.settingsViewOpen.value = true
    opts.topMoreOpen.value = false
  }

  useUnifiedKeybindings({
    appActions: [
      {
        actionId: 'app.openShortcutHelp',
        enabled: computed(() => true),
        run: openShortcutHelp,
      },
      {
        actionId: 'app.openSettings',
        enabled: computed(() => true),
        run: () => {
          if (isTheater.value) {
            hostEventBus.emit('theater:settings', { action: 'toggle' })
            return
          }
          openSettingsView()
        },
      },
      {
        actionId: 'app.openPluginManager',
        enabled: computed(() => opts.pluginUiEnabled.value),
        run: () => {
          if (!opts.pluginUiEnabled.value)
            return
          opts.openPluginManagerPanel()
        },
      },
      {
        actionId: 'app.openModelManager',
        enabled: computed(() => true),
        run: () => {
          if (isTheater.value) {
            hostEventBus.emit('theater:settings', { action: 'model' })
            return
          }
          opts.openModelManager()
        },
      },
      {
        actionId: 'app.toggleDebug',
        enabled: computed(() => (opts.debugUiEnabled ? opts.debugUiEnabled.value : true)),
        run: () => {
          if (opts.debugUiEnabled && !opts.debugUiEnabled.value)
            return
          opts.toggleDebug()
        },
      },
    ],
    holdActions: opts.holdActions,
  })

  function clearCtrlLongPressTimer(): void {
    if (ctrlLongPressTimer != null) {
      window.clearTimeout(ctrlLongPressTimer)
      ctrlLongPressTimer = null
    }
  }

  function ctrlHoldModifiersClean(e: KeyboardEvent): boolean {
    return !e.altKey && !e.shiftKey && !e.metaKey
  }

  function onCtrlHoldPointerDown(e: PointerEvent): void {
    if (e.ctrlKey && e.buttons !== 0) {
      ctrlHoldBlockedByPointer = true
      clearCtrlLongPressTimer()
    }
  }

  function onCtrlHoldHintKeydown(e: KeyboardEvent): void {
    if (e.key !== 'Control' || e.repeat)
      return
    if (!document.hasFocus() || document.visibilityState !== 'visible')
      return
    if (!ctrlHoldModifiersClean(e))
      return
    // Only show long-press hint when shortcut help is bound to Ctrl+LongPress.
    try {
      const accel = getPrimaryEffectiveAcceleratorForAction(
        loadUnifiedBindingsFile(),
        'app.openShortcutHelp',
      )
      if (accel !== 'Ctrl+LongPress')
        return
    }
    catch {
      // ignore and allow default behavior
    }
    ctrlHoldBlockedByPointer = false
    clearCtrlLongPressTimer()
    ctrlLongPressTimer = window.setTimeout(() => {
      ctrlLongPressTimer = null
      if (ctrlHoldBlockedByPointer || !document.hasFocus())
        return
      shortcutHelpOpen.value = true
    }, 1000)
  }

  function onCtrlHoldHintKeyup(e: KeyboardEvent): void {
    if (e.key === 'Control') {
      ctrlHoldBlockedByPointer = false
      clearCtrlLongPressTimer()
    }
  }

  function onHotkey(e: KeyboardEvent): void {
    if (e.key === 'Escape') {
      if (isTheater.value) {
        e.preventDefault()
        hostEventBus.emit('theater:settings', { action: 'escape' })
        return
      }
      if (opts.simplePluginManagerOpen.value) {
        e.preventDefault()
        opts.simplePluginManagerOpen.value = false
        return
      }
      if (shortcutHelpOpen.value) {
        e.preventDefault()
        shortcutHelpOpen.value = false
        return
      }
      if (opts.marketPanelVisible) {
        e.preventDefault()
        opts.closeMarketPanel()
        return
      }
      if (opts.modelManagerOpen.value) {
        e.preventDefault()
        opts.closeModelManager()
        return
      }
      if (opts.settingsViewOpen.value) {
        e.preventDefault()
        opts.settingsViewOpen.value = false
        return
      }
      if (opts.topMoreOpen.value) {
        e.preventDefault()
        opts.topMoreOpen.value = false
        return
      }
      if (opts.debugVisible) {
        e.preventDefault()
        opts.toggleDebug()
        return
      }
    }
  }

  onMounted(() => {
    window.addEventListener('keydown', onHotkey)
    window.addEventListener('keydown', onCtrlHoldHintKeydown)
    window.addEventListener('keyup', onCtrlHoldHintKeyup)
    window.addEventListener('pointerdown', onCtrlHoldPointerDown, true)
  })

  onBeforeUnmount(() => {
    window.removeEventListener('keydown', onHotkey)
    window.removeEventListener('keydown', onCtrlHoldHintKeydown)
    window.removeEventListener('keyup', onCtrlHoldHintKeyup)
    window.removeEventListener('pointerdown', onCtrlHoldPointerDown, true)
    clearCtrlLongPressTimer()
  })

  return {
    shortcutHelpOpen,
    openShortcutHelp,
    openSettingsView,
  }
}

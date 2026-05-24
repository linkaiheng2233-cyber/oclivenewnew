import type { Ref } from 'vue'
import { computed, type ComputedRef, onBeforeUnmount, onMounted, ref } from 'vue'

export interface UseGlobalHotkeysOptions {
  simplePluginManagerOpen: Ref<boolean>
  settingsViewOpen: Ref<boolean>
  topMoreOpen: Ref<boolean>
  marketPanelVisible: Ref<boolean> | ComputedRef<boolean>
  debugVisible: Ref<boolean> | ComputedRef<boolean>
  openPluginManagerPanel: () => void
  toggleDebug: () => void
  closeMarketPanel: () => void
}

export function useGlobalHotkeys(opts: UseGlobalHotkeysOptions) {
  const shortcutHelpOpen = ref(false)
  let ctrlLongPressTimer: ReturnType<typeof setTimeout> | null = null
  let ctrlHoldBlockedByPointer = false

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

  function openShortcutHelp(): void {
    shortcutHelpOpen.value = true
    opts.topMoreOpen.value = false
  }

  function onHotkey(e: KeyboardEvent): void {
    if (e.key === 'Escape') {
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
    if (e.ctrlKey && e.shiftKey && e.key.toLowerCase() === 'f') {
      e.preventDefault()
      opts.openPluginManagerPanel()
      return
    }
    if (e.ctrlKey && e.shiftKey && e.key.toLowerCase() === 's') {
      e.preventDefault()
      opts.openSettingsView()
      return
    }
    if (e.ctrlKey && e.shiftKey && e.key.toLowerCase() === 'd') {
      e.preventDefault()
      opts.toggleDebug()
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
    openSettingsView: () => {
      opts.settingsViewOpen.value = true
      opts.topMoreOpen.value = false
    },
  }
}

import type { ComputedRef, Ref } from 'vue'
import type { MainShellSettingsTab } from './useMainShellWindows'
import { useGlobalHotkeys } from '@oclive/shared/composables/useGlobalHotkeys'
import { hostEventBus } from '@oclive/shared/lib/hostEventBus'
import { computed, onBeforeUnmount, watch } from 'vue'

export function useMainShellHotkeys(options: {
  simplePluginManagerOpen: Ref<boolean>
  settingsViewOpen: Ref<boolean>
  topMoreOpen: Ref<boolean>
  marketPanelVisible: ComputedRef<boolean>
  modelManagerOpen: Ref<boolean>
  debugVisible: ComputedRef<boolean>
  pluginUiEnabled: ComputedRef<boolean>
  debugUiEnabled: ComputedRef<boolean>
  settingsUiEnabled: ComputedRef<boolean>
  voiceInputEnabled: ComputedRef<boolean>
  openPluginManagerPanel: () => void
  openModelManager: () => void
  toggleDebug: () => void
  closeMarketPanel: () => void
  closeModelManager: () => void
  settingsFocusTab: Ref<MainShellSettingsTab | null>
}) {
  let voiceHoldStarted = false
  function stopVoiceHold(): void {
    if (!voiceHoldStarted)
      return
    voiceHoldStarted = false
    hostEventBus.emit('com.oclive.voice.asr:hold', { phase: 'stop' })
  }
  watch(options.voiceInputEnabled, (enabled) => {
    if (!enabled)
      stopVoiceHold()
  }, { flush: 'sync' })
  onBeforeUnmount(stopVoiceHold)
  const {
    shortcutHelpOpen,
    openShortcutHelp,
    openSettingsView,
  } = useGlobalHotkeys({
    simplePluginManagerOpen: options.simplePluginManagerOpen,
    settingsViewOpen: options.settingsViewOpen,
    topMoreOpen: options.topMoreOpen,
    marketPanelVisible: options.marketPanelVisible,
    modelManagerOpen: options.modelManagerOpen,
    debugVisible: options.debugVisible,
    pluginUiEnabled: options.pluginUiEnabled,
    debugUiEnabled: options.debugUiEnabled,
    settingsUiEnabled: options.settingsUiEnabled,
    openPluginManagerPanel: options.openPluginManagerPanel,
    openModelManager: options.openModelManager,
    toggleDebug: options.toggleDebug,
    closeMarketPanel: options.closeMarketPanel,
    closeModelManager: options.closeModelManager,
    holdActions: [
      {
        actionId: 'voice.holdToTalk',
        enabled: options.voiceInputEnabled,
        onStart: () => {
          if (options.voiceInputEnabled.value && !voiceHoldStarted) {
            voiceHoldStarted = true
            hostEventBus.emit('com.oclive.voice.asr:hold', { phase: 'start' })
          }
        },
        onStop: stopVoiceHold,
      },
    ],
  })

  function openSettingsToGeneral(): void {
    options.settingsFocusTab.value = 'general'
    openSettingsView()
  }

  const sidePanelOpen = computed(
    () => options.settingsViewOpen.value || options.simplePluginManagerOpen.value || options.modelManagerOpen.value,
  )

  const sidePanelTab = computed<'settings' | 'plugins' | 'models'>(() => {
    if (options.settingsViewOpen.value)
      return 'settings'
    if (options.simplePluginManagerOpen.value)
      return 'plugins'
    return 'models'
  })

  return {
    shortcutHelpOpen,
    openShortcutHelp,
    openSettingsView,
    openSettingsToGeneral,
    sidePanelOpen,
    sidePanelTab,
  }
}

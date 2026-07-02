import type { ComputedRef, Ref } from 'vue'
import { computed } from 'vue'
import { useGlobalHotkeys } from '@oclive/shared/composables/useGlobalHotkeys'
import { hostEventBus } from '@oclive/shared/lib/hostEventBus'
import type { MainShellSettingsTab } from './useMainShellWindows'

export function useMainShellHotkeys(options: {
  simplePluginManagerOpen: Ref<boolean>
  settingsViewOpen: Ref<boolean>
  topMoreOpen: Ref<boolean>
  marketPanelVisible: ComputedRef<boolean>
  modelManagerOpen: Ref<boolean>
  debugVisible: ComputedRef<boolean>
  pluginUiEnabled: ComputedRef<boolean>
  debugUiEnabled: ComputedRef<boolean>
  openPluginManagerPanel: () => void
  openModelManager: () => void
  toggleDebug: () => void
  closeMarketPanel: () => void
  closeModelManager: () => void
  settingsFocusTab: Ref<MainShellSettingsTab | null>
}) {
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
    openPluginManagerPanel: options.openPluginManagerPanel,
    openModelManager: options.openModelManager,
    toggleDebug: options.toggleDebug,
    closeMarketPanel: options.closeMarketPanel,
    closeModelManager: options.closeModelManager,
    holdActions: [
      {
        actionId: 'voice.holdToTalk',
        enabled: computed(() => true),
        onStart: () => hostEventBus.emit('com.oclive.voice.asr:hold', { phase: 'start' }),
        onStop: () => hostEventBus.emit('com.oclive.voice.asr:hold', { phase: 'stop' }),
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

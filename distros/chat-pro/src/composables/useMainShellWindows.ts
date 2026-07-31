import type { usePluginMarketStore } from '@oclive/shared/stores/pluginMarketStore'
import { useModelManagerWindow } from '@oclive/shared/composables/useModelManagerWindow'
import { usePluginManagerWindow } from '@oclive/shared/composables/usePluginManagerWindow'
import { ref, watch } from 'vue'

export type MainShellSettingsTab = 'general' | 'plugins' | 'storage'

export function useMainShellWindows(options: {
  pluginMarketStore: ReturnType<typeof usePluginMarketStore>
}) {
  const { pluginMarketStore } = options
  const topMoreOpen = ref(false)
  const settingsViewOpen = ref(false)
  const settingsFocusTab = ref<MainShellSettingsTab | null>(null)

  const {
    simplePluginManagerOpen,
    pluginsPanelSubview,
    openPluginManagerPanel,
    openSimplePluginManager,
    openPluginMarket,
  } = usePluginManagerWindow({
    closeMoreMenu: () => {
      topMoreOpen.value = false
    },
  })

  const {
    modelManagerOpen,
    openModelManager,
    closeModelManager,
  } = useModelManagerWindow({
    closeMoreMenu: () => {
      topMoreOpen.value = false
    },
  })

  watch(simplePluginManagerOpen, (open) => {
    if (open) {
      modelManagerOpen.value = false
      settingsViewOpen.value = false
    }
  })

  watch(modelManagerOpen, (open) => {
    if (open) {
      simplePluginManagerOpen.value = false
      settingsViewOpen.value = false
      pluginMarketStore.closeMarketPanel()
    }
  })

  watch(settingsViewOpen, (open) => {
    if (open) {
      simplePluginManagerOpen.value = false
      modelManagerOpen.value = false
      pluginMarketStore.closeMarketPanel()
    }
  })

  watch(() => pluginMarketStore.marketPanelVisible, (open) => {
    if (open) {
      simplePluginManagerOpen.value = false
      modelManagerOpen.value = false
    }
  })

  function closeAllSidePanels(): void {
    settingsViewOpen.value = false
    simplePluginManagerOpen.value = false
    closeModelManager()
  }

  return {
    topMoreOpen,
    settingsViewOpen,
    settingsFocusTab,
    simplePluginManagerOpen,
    pluginsPanelSubview,
    openPluginManagerPanel,
    openSimplePluginManager,
    openPluginMarket,
    modelManagerOpen,
    openModelManager,
    closeModelManager,
    closeAllSidePanels,
  }
}

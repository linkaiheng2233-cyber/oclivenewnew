import { ref, watch } from 'vue'
import { ensurePluginWorkbenchI18n } from '@oclive/shared/i18n/loadPluginWorkbench'
import { usePluginMarketStore } from '@oclive/shared/stores/pluginMarketStore'
import { useRoleStore } from '@oclive/shared/stores/roleStore'
import { useOverlayWindow } from '@oclive/shared/composables/useOverlayWindow'
import { resolveOcliveShell } from './useOcliveShell'

export type PluginsPanelSubview = 'list' | 'market'

export interface UsePluginManagerWindowOptions {
  /** Collapse the top-bar "More" menu after each plugin manager open/switch. */
  closeMoreMenu: () => void
}

/** Minimal installed-plugin list and market entry. */
export function usePluginManagerWindow(opts: UsePluginManagerWindowOptions) {
  const roleStore = useRoleStore()
  const marketStore = usePluginMarketStore()
  const pluginsPanelSubview = ref<PluginsPanelSubview>('list')
  const { open: simplePluginManagerOpen, toggle } = useOverlayWindow({
    closeMoreMenu: opts.closeMoreMenu,
    onOpen: () => {
      void ensurePluginWorkbenchI18n()
      marketStore.closeMarketPanel()
    },
  })

  function openSimplePluginManager(forceOpen = false): void {
    if (!roleStore.interactionImmersive)
      return
    toggle(forceOpen)
  }

  function openPluginManagerPanel(): void {
    openSimplePluginManager()
  }

  function openPluginMarket(): void {
    if (!roleStore.interactionImmersive)
      return
    void ensurePluginWorkbenchI18n()
    if (resolveOcliveShell() === 'tool') {
      pluginsPanelSubview.value = 'market'
      toggle(true)
      marketStore.closeMarketPanel()
    }
    else {
      simplePluginManagerOpen.value = false
      void marketStore.openMarketPanel()
    }
    opts.closeMoreMenu()
  }

  watch(simplePluginManagerOpen, (open) => {
    if (!open)
      pluginsPanelSubview.value = 'list'
  })

  return {
    simplePluginManagerOpen,
    pluginsPanelSubview,
    openPluginManagerPanel,
    openSimplePluginManager,
    openPluginMarket,
  }
}

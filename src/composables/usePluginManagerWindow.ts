import { ref } from 'vue'
import { ensurePluginWorkbenchI18n } from '../i18n/loadPluginWorkbench'
import { usePluginMarketStore } from '../stores/pluginMarketStore'

export interface UsePluginManagerWindowOptions {
  /** Collapse the top-bar "More" menu after each plugin manager open/switch. */
  closeMoreMenu: () => void
}

/** Minimal installed-plugin list and market entry. */
export function usePluginManagerWindow(opts: UsePluginManagerWindowOptions) {
  const marketStore = usePluginMarketStore()
  const simplePluginManagerOpen = ref(false)

  function openSimplePluginManager(forceOpen = false): void {
    void ensurePluginWorkbenchI18n()
    marketStore.closeMarketPanel()
    if (forceOpen) {
      simplePluginManagerOpen.value = true
    }
    else {
      simplePluginManagerOpen.value = !simplePluginManagerOpen.value
    }
    opts.closeMoreMenu()
  }

  function openPluginManagerPanel(): void {
    openSimplePluginManager()
  }

  function openPluginMarket(): void {
    void ensurePluginWorkbenchI18n()
    simplePluginManagerOpen.value = false
    void marketStore.openMarketPanel()
    opts.closeMoreMenu()
  }

  return {
    simplePluginManagerOpen,
    openPluginManagerPanel,
    openSimplePluginManager,
    openPluginMarket,
  }
}

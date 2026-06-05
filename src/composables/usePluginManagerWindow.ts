import { ensurePluginWorkbenchI18n } from '../i18n/loadPluginWorkbench'
import { usePluginMarketStore } from '../stores/pluginMarketStore'
import { useOverlayWindow } from './useOverlayWindow'

export interface UsePluginManagerWindowOptions {
  /** Collapse the top-bar "More" menu after each plugin manager open/switch. */
  closeMoreMenu: () => void
}

/** Minimal installed-plugin list and market entry. */
export function usePluginManagerWindow(opts: UsePluginManagerWindowOptions) {
  const marketStore = usePluginMarketStore()
  const { open: simplePluginManagerOpen, toggle } = useOverlayWindow({
    closeMoreMenu: opts.closeMoreMenu,
    onOpen: () => {
      void ensurePluginWorkbenchI18n()
      marketStore.closeMarketPanel()
    },
  })

  function openSimplePluginManager(forceOpen = false): void {
    toggle(forceOpen)
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

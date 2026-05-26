import { ref } from 'vue'
import { ensurePluginWorkbenchI18n } from '../i18n/loadPluginWorkbench'
import { usePluginMarketStore } from '../stores/pluginMarketStore'

export interface UsePluginManagerWindowOptions {
  /** 每次打开/切换插件管理入口后收起顶栏「更多」 */
  closeMoreMenu: () => void
}

/** 极简已安装插件列表与市场入口。 */
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

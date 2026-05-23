import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { usePluginMarketStore } from '../stores/pluginMarketStore'
import { usePluginTraceStore } from '../stores/pluginTraceStore'

export interface UsePluginManagerWindowOptions {
  /** 每次打开/切换插件管理入口后收起顶栏「更多」 */
  closeMoreMenu: () => void
}

/**
 * 极简插件管理窗（唯一入口）。V1/V2/架构图面板保留源码但不在主应用挂载。
 */
export function usePluginManagerWindow(opts: UsePluginManagerWindowOptions) {
  const { t } = useI18n()
  const traceStore = usePluginTraceStore()
  const marketStore = usePluginMarketStore()
  const simplePluginManagerOpen = ref(false)

  const pluginManagerMoreBtnLabel = computed(() => t('app.more.pluginBtnSimple'))

  const settingsEntryMoreHelp = computed(() => t('app.more.settingsTileHelpSimple'))

  function openPluginManagerPanel(): void {
    traceStore.closePanel()
    simplePluginManagerOpen.value = !simplePluginManagerOpen.value
    opts.closeMoreMenu()
  }

  function openPluginMarket(): void {
    simplePluginManagerOpen.value = false
    traceStore.closePanel()
    marketStore.openMarketPanel()
    opts.closeMoreMenu()
  }

  watch(
    () => traceStore.simpleManagerOpenNonce,
    () => {
      if (traceStore.simpleManagerOpenNonce > 0) {
        traceStore.closePanel()
        simplePluginManagerOpen.value = true
      }
    },
  )

  return {
    simplePluginManagerOpen,
    openPluginManagerPanel,
    openPluginMarket,
    pluginManagerMoreBtnLabel,
    settingsEntryMoreHelp,
  }
}

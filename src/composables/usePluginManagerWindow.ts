import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { usePluginStore } from "../stores/pluginStore";

export interface UsePluginManagerWindowOptions {
  /** 每次打开/切换插件管理入口后收起顶栏「更多」 */
  closeMoreMenu: () => void;
}

/**
 * 极简插件管理窗（唯一入口）。V1/V2/架构图面板保留源码但不在主应用挂载。
 */
export function usePluginManagerWindow(opts: UsePluginManagerWindowOptions) {
  const { t } = useI18n();
  const pluginStore = usePluginStore();
  const simplePluginManagerOpen = ref(false);

  const pluginManagerMoreBtnLabel = computed(() => t("app.more.pluginBtnSimple"));

  const settingsEntryMoreHelp = computed(() => t("app.more.settingsTileHelpSimple"));

  function openPluginManagerPanel(): void {
    pluginStore.closePanel();
    simplePluginManagerOpen.value = !simplePluginManagerOpen.value;
    opts.closeMoreMenu();
  }

  function openPluginMarket(): void {
    simplePluginManagerOpen.value = false;
    pluginStore.closePanel();
    void pluginStore.openMarketPanel();
    opts.closeMoreMenu();
  }

  watch(
    () => pluginStore.simpleManagerOpenNonce,
    () => {
      if (pluginStore.simpleManagerOpenNonce > 0) {
        pluginStore.closePanel();
        simplePluginManagerOpen.value = true;
      }
    },
  );

  return {
    simplePluginManagerOpen,
    openPluginManagerPanel,
    openPluginMarket,
    pluginManagerMoreBtnLabel,
    settingsEntryMoreHelp,
  };
}

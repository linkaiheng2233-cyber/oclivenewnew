import { computed } from "vue";
import { moreMenuTileHelpText } from "../lib/pluginManagerEntryCopy";
import { useUiStore } from "../stores/uiStore";

export interface UsePluginManagerWindowOptions {
  /** 每次打开/切换插件管理入口后收起顶栏「更多」 */
  closeMoreMenu: () => void;
}

/**
 * 顶栏「更多」设置入口 tile 的 HelpHint 文案（V2 面板已迁入设置中心嵌入区）。
 */
export function usePluginManagerWindow(_opts: UsePluginManagerWindowOptions) {
  const uiStore = useUiStore();

  const settingsEntryMoreHelp = computed(() =>
    moreMenuTileHelpText(uiStore.experimentalPluginManagerV2),
  );

  return {
    settingsEntryMoreHelp,
  };
}

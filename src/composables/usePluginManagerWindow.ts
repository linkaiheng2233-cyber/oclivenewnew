import { computed, ref, watch } from "vue";
import {
  moreMenuPluginButtonLabel,
  moreMenuTileHelpText,
} from "../lib/pluginManagerEntryCopy";
import { usePluginStore } from "../stores/pluginStore";
import { useUiStore } from "../stores/uiStore";

export interface UsePluginManagerWindowOptions {
  /** 每次打开/切换插件管理入口后收起顶栏「更多」 */
  closeMoreMenu: () => void;
  /** 从设置页打开 V2 预览时额外关闭设置窗 */
  closeSettingsView?: () => void;
}

/**
 * V1 专业面板（pluginStore.panelVisible）与 V2 预览窗（pluginManagerV2Open）的打开逻辑。
 * 是否走 V2 由 `uiStore.experimentalPluginManagerV2`（持久化）决定。
 */
export function usePluginManagerWindow(opts: UsePluginManagerWindowOptions) {
  const uiStore = useUiStore();
  const pluginStore = usePluginStore();
  const pluginManagerV2Open = ref(false);

  const pluginManagerMoreBtnLabel = computed(() =>
    moreMenuPluginButtonLabel(uiStore.experimentalPluginManagerV2),
  );

  const settingsEntryMoreHelp = computed(() =>
    moreMenuTileHelpText(uiStore.experimentalPluginManagerV2),
  );

  function openPluginManagerPanel(): void {
    if (uiStore.experimentalPluginManagerV2) {
      pluginStore.closePanel();
      pluginManagerV2Open.value = !pluginManagerV2Open.value;
      opts.closeMoreMenu();
      return;
    }
    pluginManagerV2Open.value = false;
    if (pluginStore.panelVisible) {
      pluginStore.closePanel();
    } else {
      void pluginStore.openPanel();
    }
    opts.closeMoreMenu();
  }

  function openPluginManagerV2Preview(): void {
    pluginStore.closePanel();
    pluginManagerV2Open.value = true;
    opts.closeMoreMenu();
    opts.closeSettingsView?.();
  }

  watch(
    () => uiStore.experimentalPluginManagerV2,
    (v) => {
      if (!v) {
        pluginManagerV2Open.value = false;
      }
    },
  );

  return {
    pluginManagerV2Open,
    openPluginManagerPanel,
    openPluginManagerV2Preview,
    pluginManagerMoreBtnLabel,
    settingsEntryMoreHelp,
  };
}

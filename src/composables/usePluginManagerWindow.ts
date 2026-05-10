import { computed, ref, watch } from "vue";
import {
  moreMenuPluginButtonLabel,
  moreMenuTileHelpText,
} from "../lib/pluginManagerEntryCopy";
import { usePluginStore, type PluginPanelMainTab } from "../stores/pluginStore";
import { useUiStore } from "../stores/uiStore";

export interface UsePluginManagerWindowOptions {
  /** 每次打开/切换插件管理入口后收起顶栏「更多」 */
  closeMoreMenu: () => void;
}

/**
 * V1 专业面板（pluginStore.panelVisible）与 V2 预览窗（pluginManagerV2Open）的打开逻辑。
 * 是否走 V2 由 `uiStore.experimentalPluginManagerV2`（持久化）决定。
 */
export function usePluginManagerWindow(opts: UsePluginManagerWindowOptions) {
  const uiStore = useUiStore();
  const pluginStore = usePluginStore();
  const pluginManagerV2Open = ref(false);
  const pluginMarketV2Open = ref(false);

  const pluginManagerMoreBtnLabel = computed(() =>
    moreMenuPluginButtonLabel(uiStore.experimentalPluginManagerV2),
  );

  const settingsEntryMoreHelp = computed(() =>
    moreMenuTileHelpText(uiStore.experimentalPluginManagerV2),
  );

  function openPluginManagerPanel(): void {
    if (uiStore.experimentalPluginManagerV2) {
      pluginStore.closePanel();
      pluginStore.closeMarketPanel();
      pluginManagerV2Open.value = !pluginManagerV2Open.value;
      if (pluginManagerV2Open.value) {
        pluginMarketV2Open.value = false;
      }
      opts.closeMoreMenu();
      return;
    }
    pluginManagerV2Open.value = false;
    pluginMarketV2Open.value = false;
    pluginStore.closeMarketPanel();
    if (pluginStore.panelVisible) {
      pluginStore.closePanel();
    } else {
      void pluginStore.openPanel();
    }
    opts.closeMoreMenu();
  }

  function openPluginMarketPanel(): void {
    if (uiStore.experimentalPluginManagerV2) {
      pluginStore.closePanel();
      pluginStore.closeMarketPanel();
      pluginMarketV2Open.value = !pluginMarketV2Open.value;
      if (pluginMarketV2Open.value) {
        pluginManagerV2Open.value = false;
      }
      opts.closeMoreMenu();
      return;
    }
    pluginManagerV2Open.value = false;
    pluginMarketV2Open.value = false;
    pluginStore.closePanel();
    if (pluginStore.marketPanelVisible) {
      pluginStore.closeMarketPanel();
    } else {
      void pluginStore.openMarketPanel();
    }
    opts.closeMoreMenu();
  }

  function openPluginManagerV2Preview(): void {
    pluginStore.closePanel();
    pluginStore.closeMarketPanel();
    pluginManagerV2Open.value = true;
    pluginMarketV2Open.value = false;
    opts.closeMoreMenu();
  }

  /**
   * 从设置等入口「确保打开」管理面板：不 toggle 关闭已打开的 V1 目标 Tab。
   * V2 实验开启且无具体 Tab 时打开 V2；若指定 Tab（含 backends）则关闭 V2 并打开 V1 对应页。
   */
  function ensureOpenPluginManagerPanel(tab?: PluginPanelMainTab): void {
    pluginMarketV2Open.value = false;
    if (uiStore.experimentalPluginManagerV2) {
      if (tab === undefined) {
        pluginStore.closePanel();
        pluginStore.closeMarketPanel();
        pluginManagerV2Open.value = true;
        opts.closeMoreMenu();
        return;
      }
      pluginManagerV2Open.value = false;
      void pluginStore.openPanel(tab);
      opts.closeMoreMenu();
      return;
    }
    pluginManagerV2Open.value = false;
    void pluginStore.openPanel(tab);
    opts.closeMoreMenu();
  }

  /** 从设置等入口确保打开市场面板（不依赖 toggle 语义）。 */
  function ensureOpenPluginMarketPanel(): void {
    pluginManagerV2Open.value = false;
    if (uiStore.experimentalPluginManagerV2) {
      pluginStore.closePanel();
      pluginStore.closeMarketPanel();
      pluginMarketV2Open.value = true;
      opts.closeMoreMenu();
      return;
    }
    pluginStore.closePanel();
    void pluginStore.openMarketPanel();
    opts.closeMoreMenu();
  }

  watch(
    () => uiStore.experimentalPluginManagerV2,
    (v) => {
      if (!v) {
        pluginManagerV2Open.value = false;
        pluginMarketV2Open.value = false;
      }
    },
  );

  return {
    pluginManagerV2Open,
    pluginMarketV2Open,
    openPluginManagerPanel,
    openPluginMarketPanel,
    openPluginManagerV2Preview,
    ensureOpenPluginManagerPanel,
    ensureOpenPluginMarketPanel,
    pluginManagerMoreBtnLabel,
    settingsEntryMoreHelp,
  };
}

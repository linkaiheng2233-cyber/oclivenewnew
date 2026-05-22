import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { usePluginStore } from "../stores/pluginStore";
import { useUiStore } from "../stores/uiStore";

export interface UsePluginManagerWindowOptions {
  /** 每次打开/切换插件管理入口后收起顶栏「更多」 */
  closeMoreMenu: () => void;
  /** 从设置页打开高级面板时额外关闭设置窗 */
  closeSettingsView?: () => void;
}

/** 从旧 Pinia 键 `experimentalPluginManagerV2` 迁移到 `advancedPluginManagement`。 */
function migratePluginManagerPersistFlag(): void {
  try {
    const raw = localStorage.getItem("ui");
    if (!raw) return;
    const parsed = JSON.parse(raw) as Record<string, unknown>;
    if (
      "experimentalPluginManagerV2" in parsed &&
      !("advancedPluginManagement" in parsed)
    ) {
      parsed.advancedPluginManagement = parsed.experimentalPluginManagerV2;
      localStorage.setItem("ui", JSON.stringify(parsed));
    }
  } catch {
    /* ignore */
  }
}

/**
 * 简化插件列表（默认）与 V1 高级面板（含架构图）的打开逻辑。
 * 由 `uiStore.advancedPluginManagement`（持久化）决定。
 */
export function usePluginManagerWindow(opts: UsePluginManagerWindowOptions) {
  migratePluginManagerPersistFlag();

  const { t } = useI18n();
  const uiStore = useUiStore();
  const pluginStore = usePluginStore();
  const simplePluginManagerOpen = ref(false);

  const pluginManagerMoreBtnLabel = computed(() =>
    uiStore.advancedPluginManagement
      ? t("app.more.pluginBtnAdvanced")
      : t("app.more.pluginBtnSimple"),
  );

  const settingsEntryMoreHelp = computed(() =>
    uiStore.advancedPluginManagement
      ? t("app.more.settingsTileHelpAdvanced")
      : t("app.more.settingsTileHelpSimple"),
  );

  function openPluginManagerPanel(): void {
    if (uiStore.advancedPluginManagement) {
      simplePluginManagerOpen.value = false;
      if (pluginStore.panelVisible) {
        pluginStore.closePanel();
      } else {
        void pluginStore.openPanel();
      }
      opts.closeMoreMenu();
      return;
    }
    pluginStore.closePanel();
    simplePluginManagerOpen.value = !simplePluginManagerOpen.value;
    opts.closeMoreMenu();
  }

  function openAdvancedPluginManager(): void {
    simplePluginManagerOpen.value = false;
    pluginStore.closePanel();
    void pluginStore.openPanel("graph");
    opts.closeMoreMenu();
    opts.closeSettingsView?.();
  }

  function openPluginMarket(): void {
    simplePluginManagerOpen.value = false;
    pluginStore.closePanel();
    void pluginStore.openMarketPanel();
    opts.closeMoreMenu();
  }

  watch(
    () => uiStore.advancedPluginManagement,
    (v) => {
      if (v) {
        simplePluginManagerOpen.value = false;
      }
    },
  );

  return {
    simplePluginManagerOpen,
    openPluginManagerPanel,
    openAdvancedPluginManager,
    openPluginMarket,
    pluginManagerMoreBtnLabel,
    settingsEntryMoreHelp,
  };
}

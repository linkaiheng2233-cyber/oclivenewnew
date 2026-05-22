import { defineStore } from "pinia";

export const useUiStore = defineStore(
  "ui",
  {
    state: () => ({
      /** 叙事场景 id；与 DB `user_presence_scene` 对齐由 App `applyResolvedNarrativeScene` 写入，避免与后端长期分叉 */
      sceneId: "home",
      /**
       * 高级插件管理：开启后 Ctrl+Shift+F 打开 V1 专业面板（含架构图）；
       * 关闭（默认）打开简化列表。
       */
      advancedPluginManagement: false,
      /**
       * 弱网/降级全局提示（不持久化，见 persist.pick）。
       * `kind === plugin_index_offline`：社区索引同步失败且已用本地缓存。
       */
      connectivityBanner: null as null | {
        kind: "plugin_index_offline";
        detail?: string;
      },
    }),
    getters: {
      /** @deprecated 使用 `advancedPluginManagement` */
      experimentalPluginManagerV2(state): boolean {
        return state.advancedPluginManagement;
      },
    },
    actions: {
      setScene(sceneId: string) {
        this.sceneId = sceneId;
      },
      setAdvancedPluginManagement(enabled: boolean) {
        this.advancedPluginManagement = enabled;
      },
      /** @deprecated 迁移自 experimentalPluginManagerV2 */
      setExperimentalPluginManagerV2(enabled: boolean) {
        this.setAdvancedPluginManagement(enabled);
      },
      setPluginIndexOfflineBanner(detail?: string) {
        this.connectivityBanner = {
          kind: "plugin_index_offline",
          detail: detail?.trim() || undefined,
        };
      },
      clearPluginIndexConnectivityBanner() {
        if (this.connectivityBanner?.kind === "plugin_index_offline") {
          this.connectivityBanner = null;
        }
      },
      dismissConnectivityBanner() {
        this.connectivityBanner = null;
      },
    },
    persist: {
      pick: ["sceneId", "advancedPluginManagement"],
    },
  },
);


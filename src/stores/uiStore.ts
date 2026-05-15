import { defineStore } from "pinia";

export const useUiStore = defineStore(
  "ui",
  {
    state: () => ({
      /** 叙事场景 id；与 DB `user_presence_scene` 对齐由 App `applyResolvedNarrativeScene` 写入，避免与后端长期分叉 */
      sceneId: "home",
      /** 灰度开关：是否优先使用 Plugin Manager V2。 */
      experimentalPluginManagerV2: false,
      /**
       * 弱网/降级全局提示（不持久化，见 persist.pick）。
       * `kind === plugin_index_offline`：社区索引同步失败且已用本地缓存。
       */
      connectivityBanner: null as null | {
        kind: "plugin_index_offline";
        detail?: string;
      },
    }),
    actions: {
      setScene(sceneId: string) {
        this.sceneId = sceneId;
      },
      setExperimentalPluginManagerV2(enabled: boolean) {
        this.experimentalPluginManagerV2 = enabled;
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
      pick: ["sceneId", "experimentalPluginManagerV2"],
    },
  },
);


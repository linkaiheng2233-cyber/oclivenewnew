import { defineStore } from "pinia";

export const useUiStore = defineStore(
  "ui",
  {
    state: () => ({
      /** 叙事场景 id；与 DB `user_presence_scene` 对齐由 App `applyResolvedNarrativeScene` 写入，避免与后端长期分叉 */
      sceneId: "home",
      /** 灰度开关：是否优先使用 Plugin Manager V2（Tauri 2 + 官方 dialog/fs 后默认可用）。 */
      experimentalPluginManagerV2: true,
    }),
    actions: {
      setScene(sceneId: string) {
        this.sceneId = sceneId;
      },
      setExperimentalPluginManagerV2(enabled: boolean) {
        this.experimentalPluginManagerV2 = enabled;
      },
    },
    persist: true,
  },
);


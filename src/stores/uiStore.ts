import { defineStore } from "pinia";

export const useUiStore = defineStore(
  "ui",
  {
    state: () => ({
      /** 叙事场景 id；与 DB `user_presence_scene` 对齐由 App `applyResolvedNarrativeScene` 写入，避免与后端长期分叉 */
      sceneId: "home",
    }),
    actions: {
      setScene(sceneId: string) {
        this.sceneId = sceneId;
      },
    },
    persist: true,
  },
);


import { defineStore } from "pinia";
import type { LanguagePref } from "../i18n";
import { resolveLocale } from "../i18n";

export const useUiStore = defineStore(
  "ui",
  {
    state: () => ({
      /** 叙事场景 id；与 DB `user_presence_scene` 对齐由 App `applyResolvedNarrativeScene` 写入，避免与后端长期分叉 */
      sceneId: "home",
      /** 灰度开关：是否优先使用 Plugin Manager V2。 */
      experimentalPluginManagerV2: false,
      /** UI 语言偏好：system 表示跟随系统语言（zh → zh-CN，否则 en-US）。 */
      languagePref: "system" as LanguagePref,
    }),
    getters: {
      effectiveLocale(state) {
        return resolveLocale(state.languagePref);
      },
    },
    actions: {
      setScene(sceneId: string) {
        this.sceneId = sceneId;
      },
      setExperimentalPluginManagerV2(enabled: boolean) {
        this.experimentalPluginManagerV2 = enabled;
      },
      setLanguagePref(pref: LanguagePref) {
        this.languagePref = pref;
      },
    },
    persist: true,
  },
);


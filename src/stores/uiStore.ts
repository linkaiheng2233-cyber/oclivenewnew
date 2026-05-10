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
      /**
       * 设置中心「开发者总闸」：关闭时侧栏仅保留 V1 等价分区；开启后额外显示专家模型、V2 Hub、实验开关、系统开发者页、Agent 调试等。
       */
      settingsDeveloperMaster: false,
      /** UI 语言偏好：system 表示跟随系统语言（zh → zh-CN，否则 en-US）。 */
      languagePref: "system" as LanguagePref,
      /**
       * 宿主请求设置打开后切换到的侧栏 id（`SETTINGS_NAV.*` 字符串值）。
       * 由 `SettingsView` 在 `visible` 时消费一次后清空。
       */
      settingsPendingNavId: null as string | null,
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
      setSettingsDeveloperMaster(enabled: boolean) {
        this.settingsDeveloperMaster = enabled;
      },
      setLanguagePref(pref: LanguagePref) {
        this.languagePref = pref;
      },
      requestSettingsNav(navId: string) {
        this.settingsPendingNavId = navId;
      },
      consumeSettingsPendingNavId(): string | null {
        const v = this.settingsPendingNavId;
        this.settingsPendingNavId = null;
        return v;
      },
    },
    persist: true,
  },
);


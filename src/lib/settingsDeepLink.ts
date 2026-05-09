import type { PluginPanelMainTab } from "../stores/pluginStore";

/**
 * 设置侧栏「打开外部面板」载荷；由 `SettingsView` 发出、`App.vue` 消费。
 * 约定：宿主在派发时先关闭设置窗，再执行打开逻辑。
 */
export type SettingsDeepLink =
  | { kind: "local_models" }
  | { kind: "plugin_manager"; tab?: PluginPanelMainTab }
  | { kind: "plugin_market" }
  | { kind: "expert_workbench"; draftMode?: "effective" | "role_default" }
  | { kind: "debug_panel" };

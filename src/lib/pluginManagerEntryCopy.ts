import { i18n } from "../i18n";

/**
 * 插件管理入口（V1 专业面板 / V2 预览）与设置里实验开关相关的**用户可见文案**单源。
 * 避免 App / 设置 / 快捷键说明三处漂移。
 *
 * 含 `<strong>` 的字符串仅用于设置页静态说明（`v-html`），勿拼接用户输入。
 */

function t(key: string): string {
  return String(i18n.global.t(key));
}

/** 设置 · 常规首段（允许 `<strong>`，由调用方 `v-html` 渲染） */
export function settingsGeneralLeadHtml(): string {
  return t("pluginManager.entry.settingsGeneralLeadHtml");
}

/** 设置 ·「启用新版插件管理界面」说明（`v-html`） */
export function settingsExperimentalToggleDescriptionHtml(): string {
  return t("pluginManager.entry.settingsExperimentalToggleDescriptionHtml");
}

/** 设置 · 常规 ·「快捷」旁 HelpHint */
export function settingsShortcutsHelpHint(): string {
  return t("pluginManager.entry.settingsShortcutsHelpHint");
}

/** 设置 · 实验性功能 区块标题旁 HelpHint */
export function settingsExperimentalSectionHelpHint(): string {
  return t("pluginManager.entry.settingsExperimentalSectionHelpHint");
}

/** 设置 ·「启用新版插件管理界面」勾选说明 */
export function settingsExperimentalToggleDescription(): string {
  return t("pluginManager.entry.settingsExperimentalToggleDescriptionHtml").replaceAll(
    /<\/?strong>/g,
    "",
  );
}

export function settingsOpenV2PreviewButtonLabel(): string {
  return t("pluginManager.entry.settingsOpenV2PreviewButtonLabel");
}

/** 顶栏「更多」里插件入口按钮文案 */
export function moreMenuPluginButtonLabel(experimentalV2: boolean): string {
  return experimentalV2
    ? t("pluginManager.moreMenu.pluginButtonLabel.v2")
    : t("pluginManager.moreMenu.pluginButtonLabel.v1");
}

/** 顶栏「更多」· 设置入口 tile 的 HelpHint 全文 */
export function moreMenuTileHelpText(experimentalV2: boolean): string {
  return experimentalV2
    ? t("pluginManager.moreMenu.tileHelpText.v2")
    : t("pluginManager.moreMenu.tileHelpText.v1");
}

/** 快捷键说明对话框中 Ctrl+Shift+F 一行 */
export function shortcutHelpCtrlShiftFDescription(experimentalV2: boolean): string {
  return experimentalV2
    ? t("pluginManager.shortcuts.ctrlShiftFDescription.v2")
    : t("pluginManager.shortcuts.ctrlShiftFDescription.v1");
}

/** 快捷键说明对话框中 Ctrl+Shift+A 一行。 */
export function shortcutHelpCtrlShiftADescription(): string {
  return t("pluginManager.shortcuts.ctrlShiftADescription");
}

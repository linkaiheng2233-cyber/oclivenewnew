/**
 * OCLive Manager（市场）与插件管理入口文案单源。
 * 避免 App / 设置 / 快捷键说明三处漂移。
 *
 * 含 `<strong>` 的字符串仅用于设置页静态说明（`v-html`），勿拼接用户输入。
 */

const TAIL_DEBUG_SHORTCUT = " Ctrl+Shift+D 开关调试面板。";

/** 设置 · 常规首段（允许 `<strong>`，由调用方 `v-html` 渲染） */
export function settingsGeneralLeadHtml(): string {
  return (
    "顶栏<strong>「更多」</strong>集中设置入口；打开设置可用 <strong>Ctrl+Shift+S</strong>；" +
    "<strong>Ctrl+Shift+A</strong> 打开 <strong>OCLive Manager（插件市场）</strong>；" +
    "<strong>Ctrl+Shift+F</strong> 打开 <strong>插件管理（专业模式）</strong>。"
  );
}

/** 设置 ·「启用新版插件管理界面」说明（`v-html`） */
export function settingsExperimentalToggleDescriptionHtml(): string {
  return (
    "开启后，可从设置内打开 <strong>V2 预览</strong>；默认快捷键仍保留：<strong>Ctrl+Shift+A</strong> 打开 OCLive Manager，<strong>Ctrl+Shift+F</strong> 打开插件管理。" +
    "需要「开发者调试」等完整能力时，请在 V2 内进入<strong>专业模式（V1）</strong>；也可关闭本项恢复默认。"
  );
}

/** 设置 · 常规 ·「快捷」旁 HelpHint */
export function settingsShortcutsHelpHint(): string {
  return "Ctrl+Shift+S 打开设置；Ctrl+Shift+A 打开 OCLive Manager（插件市场）；Ctrl+Shift+F 打开插件管理；Ctrl+Shift+D 开关调试面板。";
}

/** 设置 · 实验性功能 区块标题旁 HelpHint */
export function settingsExperimentalSectionHelpHint(): string {
  return "灰度入口：用于预览新版插件管理界面（V2）。若当前构建未集成 V2，会继续使用现有专业模式。";
}

/** 设置 ·「启用新版插件管理界面」勾选说明 */
export function settingsExperimentalToggleDescription(): string {
  return (
    "开启后，可从设置内打开 V2 预览；默认快捷键仍保留：Ctrl+Shift+A 打开 OCLive Manager，Ctrl+Shift+F 打开插件管理。" +
    "需要「开发者调试」等完整能力时，请在 V2 内进入专业模式（V1）；也可关闭本项恢复默认。"
  );
}

export function settingsOpenV2PreviewButtonLabel(): string {
  return "打开插件管理 V2 预览";
}

/** 顶栏「更多」里插件管理入口按钮文案 */
export function moreMenuPluginManageButtonLabel(): string {
  return "插件管理";
}

/** 顶栏「更多」里 OCLive Manager（市场）入口按钮文案 */
export function moreMenuOcliveManagerButtonLabel(): string {
  return "OCLive Manager";
}

/** 顶栏「更多」· 设置入口 tile 的 HelpHint 全文 */
export function moreMenuTileHelpText(experimentalV2: boolean): string {
  if (experimentalV2) {
    return (
      "将快捷键说明、设置页、管理器入口集中到同一处。快捷键：Ctrl+Shift+S 打开设置；" +
      "Ctrl+Shift+A 打开 OCLive Manager（插件市场）；Ctrl+Shift+F 打开插件管理；可在设置中开启 V2 预览。" +
      TAIL_DEBUG_SHORTCUT
    );
  }
  return (
    "将快捷键说明、设置页、管理器入口集中到同一处。快捷键：Ctrl+Shift+S 打开设置；" +
    "Ctrl+Shift+A 打开 OCLive Manager（插件市场）；Ctrl+Shift+F 打开插件管理（含开发者调试）。" +
    TAIL_DEBUG_SHORTCUT
  );
}

/** 快捷键说明对话框中 Ctrl+Shift+A 一行 */
export function shortcutHelpCtrlShiftADescription(): string {
  return "打开 OCLive Manager（插件市场）；可从 GitHub 共享索引安装/更新插件";
}

/** 快捷键说明对话框中 Ctrl+Shift+F 一行 */
export function shortcutHelpCtrlShiftFDescription(experimentalV2: boolean): string {
  return experimentalV2
    ? "打开插件管理（专业模式 V1）；V2 预览可在设置页单独打开"
    : "打开插件管理（专业模式 V1，含后端、插槽与开发者调试）";
}

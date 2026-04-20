/**
 * 插件管理入口（V1 专业面板 / V2 预览）与设置里实验开关相关的**用户可见文案**单源。
 * 避免 App / 设置 / 快捷键说明三处漂移。
 *
 * 含 `<strong>` 的字符串仅用于设置页静态说明（`v-html`），勿拼接用户输入。
 */

const TAIL_DEBUG_SHORTCUT = " Ctrl+Shift+D 开关调试面板。";

/** 设置 · 常规首段（允许 `<strong>`，由调用方 `v-html` 渲染） */
export function settingsGeneralLeadHtml(): string {
  return (
    "顶栏<strong>「更多」</strong>集中设置入口；打开设置可用 <strong>Ctrl+Shift+S</strong>；" +
    "<strong>Ctrl+Shift+F</strong> 打开插件管理（未勾选下方「V2 预览」时为<strong>专业模式（V1）</strong>；" +
    "勾选后同一快捷键为<strong>V2 预览</strong>，V1 可从 V2 内入口打开）。"
  );
}

/** 设置 ·「启用新版插件管理界面」说明（`v-html`） */
export function settingsExperimentalToggleDescriptionHtml(): string {
  return (
    "开启后，<strong>Ctrl+Shift+F</strong> 与顶栏「更多」里的插件管理入口将<strong>打开并切换 V2 预览</strong>（设置会记住此项）。" +
    "需要「开发者调试」等完整能力时，请在 V2 内进入<strong>专业模式（V1）</strong>；也可关闭本项恢复默认。"
  );
}

/** 设置 · 常规 ·「快捷」旁 HelpHint */
export function settingsShortcutsHelpHint(): string {
  return "Ctrl+Shift+S 打开设置；Ctrl+Shift+F 打开插件管理（V1/V2 由下方实验性勾选决定）；Ctrl+Shift+D 开关调试面板。";
}

/** 设置 · 实验性功能 区块标题旁 HelpHint */
export function settingsExperimentalSectionHelpHint(): string {
  return "灰度入口：用于预览新版插件管理界面（V2）。若当前构建未集成 V2，会继续使用现有专业模式。";
}

/** 设置 ·「启用新版插件管理界面」勾选说明 */
export function settingsExperimentalToggleDescription(): string {
  return (
    "开启后，Ctrl+Shift+F 与顶栏「更多」里的插件管理入口将打开并切换 V2 预览（设置会记住此项）。" +
    "需要「开发者调试」等完整能力时，请在 V2 内进入专业模式（V1）；也可关闭本项恢复默认。"
  );
}

export function settingsOpenV2PreviewButtonLabel(): string {
  return "打开插件管理 V2 预览";
}

/** 顶栏「更多」里插件入口按钮文案 */
export function moreMenuPluginButtonLabel(experimentalV2: boolean): string {
  return experimentalV2 ? "插件管理（V2）" : "插件与后端（V1）";
}

/** 顶栏「更多」· 设置入口 tile 的 HelpHint 全文 */
export function moreMenuTileHelpText(experimentalV2: boolean): string {
  if (experimentalV2) {
    return (
      "将快捷键说明、设置页、插件管理集中到同一处。快捷键：Ctrl+Shift+S 打开设置；" +
      "Ctrl+Shift+F 与下方按钮打开插件管理（V2 预览）；在设置中关闭「V2 预览」可恢复专业模式（V1）。" +
      TAIL_DEBUG_SHORTCUT
    );
  }
  return (
    "将快捷键说明、设置页、插件与后端管理集中到同一处。快捷键：Ctrl+Shift+S 打开设置；" +
    "Ctrl+Shift+F 打开专业模式（V1）插件与后端管理（含开发者调试）。" +
    TAIL_DEBUG_SHORTCUT
  );
}

/** 快捷键说明对话框中 Ctrl+Shift+F 一行 */
export function shortcutHelpCtrlShiftFDescription(experimentalV2: boolean): string {
  return experimentalV2
    ? "打开插件管理（V2 预览）；关闭设置中的「V2 预览」后恢复为专业模式（V1）"
    : "打开专业模式（V1）插件与后端管理（含界面插件 · 开发者调试）";
}

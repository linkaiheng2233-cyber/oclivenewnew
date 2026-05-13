import { i18n } from "../i18n";
import type { SettingsTier } from "./settingsNavKeys";

/**
 * 设置分级（L1–L4）徽章与说明：单源供侧栏与右栏引用，避免与插件入口文案漂移。
 * 文案键：`settings.tiers.*`（见 locales）。
 */

function t(key: string): string {
  return String(i18n.global.t(key));
}

export function settingsTierBadge(tier: SettingsTier): string {
  return t(`settings.tiers.${tier}.badge`);
}

export function settingsTierDescription(tier: SettingsTier): string {
  return t(`settings.tiers.${tier}.description`);
}

/** 深链按钮下方统一说明（关闭设置后打开独立面板） */
export function settingsDeepLinkFooterNote(): string {
  return t("settings.nav.deepLinkFooterNote");
}

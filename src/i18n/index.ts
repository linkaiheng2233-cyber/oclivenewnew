import { createI18n } from "vue-i18n";
import { applyShortcutModTokens } from "../lib/shortcutDisplay";

export type AppLocale = "zh-CN" | "en-US";
export type LanguagePref = "system" | AppLocale;

const emptyMessages = { "zh-CN": {}, "en-US": {} } as Record<AppLocale, Record<string, unknown>>;

const loadedLocales = new Set<AppLocale>();

export function detectSystemLocale(): AppLocale {
  if (typeof navigator === "undefined") return "en-US";
  const langs = Array.isArray(navigator.languages)
    ? navigator.languages
    : [navigator.language];
  for (const lang of langs) {
    if (typeof lang === "string" && lang.toLowerCase().startsWith("zh")) {
      return "zh-CN";
    }
  }
  return "en-US";
}

export function resolveLocale(pref: LanguagePref): AppLocale {
  return pref === "system" ? detectSystemLocale() : pref;
}

export const i18n = createI18n({
  legacy: false,
  locale: detectSystemLocale(),
  fallbackLocale: "en-US",
  messages: emptyMessages,
  missingWarn: false,
  fallbackWarn: false,
  /** Locale strings intentionally use HTML in a few places; silence dev/test noise. */
  warnHtmlMessage: false,
});

/** 合并指定语言的完整文案（幂等；按需 dynamic import）。 */
export async function prepareI18nForLocale(locale: AppLocale): Promise<void> {
  if (loadedLocales.has(locale)) return;
  if (locale === "zh-CN") {
    const m = await import("./locales/zh-CN");
    i18n.global.mergeLocaleMessage("zh-CN", applyShortcutModTokens(m.zhCN));
  } else {
    const m = await import("./locales/en-US");
    i18n.global.mergeLocaleMessage("en-US", applyShortcutModTokens(m.enUS));
  }
  loadedLocales.add(locale);
}

export async function setAppLocale(locale: AppLocale): Promise<void> {
  await prepareI18nForLocale(locale);
  i18n.global.locale.value = locale;
}

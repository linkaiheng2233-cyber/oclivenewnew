import { createI18n } from "vue-i18n";
import { enUS } from "./locales/en-US";
import { zhCN } from "./locales/zh-CN";

export type AppLocale = "zh-CN" | "en-US";
export type LanguagePref = "system" | AppLocale;

export const messages = {
  "en-US": enUS,
  "zh-CN": zhCN,
} as const;

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
  messages,
});

export function setAppLocale(locale: AppLocale): void {
  // `global.locale` is a Ref in composition mode (legacy: false).
  i18n.global.locale.value = locale;
}


import { createI18n } from "vue-i18n";
import zhCN from "./locales/zh-CN";
import enUS from "./locales/en-US";

export const LOCALE_PREF_KEY = "oclive.appLocale";

/** Stored preference; `"system"` resolves to browser language when applied. */
export type LocalePreference = "system" | "zh-CN" | "en-US";

export function getBrowserLocaleTag(): "zh-CN" | "en-US" {
  if (typeof navigator === "undefined") return "zh-CN";
  const raw = (navigator.languages?.[0] ?? navigator.language ?? "zh-CN").toLowerCase();
  return raw.startsWith("zh") ? "zh-CN" : "en-US";
}

export function resolveLocaleTag(pref: LocalePreference): "zh-CN" | "en-US" {
  if (pref === "system") return getBrowserLocaleTag();
  return pref;
}

export function getLocalePreference(): LocalePreference {
  try {
    const v = localStorage.getItem(LOCALE_PREF_KEY);
    if (v === "system" || v === "zh-CN" || v === "en-US") return v;
  } catch {
    /* ignore */
  }
  return "system";
}

const initialPref = getLocalePreference();

export const i18n = createI18n({
  legacy: false,
  locale: resolveLocaleTag(initialPref),
  fallbackLocale: "zh-CN",
  messages: {
    "zh-CN": zhCN,
    "en-US": enUS,
  },
});

export function setLocalePreference(pref: LocalePreference): void {
  try {
    localStorage.setItem(LOCALE_PREF_KEY, pref);
  } catch {
    /* ignore */
  }
  i18n.global.locale.value = resolveLocaleTag(pref);
}

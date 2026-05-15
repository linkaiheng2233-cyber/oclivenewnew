/** localStorage：置为 `"1"` 时跳过 Sentry 初始化（与设置页「禁用崩溃上报」同步）。 */
export const SENTRY_OPT_OUT_STORAGE_KEY = "oclive.telemetry.sentryOptOut";

export function isSentryOptOut(): boolean {
  try {
    return globalThis.localStorage?.getItem(SENTRY_OPT_OUT_STORAGE_KEY) === "1";
  } catch {
    return false;
  }
}

export function setSentryOptOut(optOut: boolean): void {
  try {
    if (optOut) {
      globalThis.localStorage?.setItem(SENTRY_OPT_OUT_STORAGE_KEY, "1");
    } else {
      globalThis.localStorage?.removeItem(SENTRY_OPT_OUT_STORAGE_KEY);
    }
  } catch {
    /* ignore quota / private mode */
  }
}

/** 构建期注入 DSN 且用户未选择退出时，才加载 Sentry。 */
export function shouldLoadSentry(dsn: string | undefined): boolean {
  return typeof dsn === "string" && dsn.length > 0 && !isSentryOptOut();
}

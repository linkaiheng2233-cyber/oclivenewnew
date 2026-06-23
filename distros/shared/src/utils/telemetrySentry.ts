/** localStorage: set to `"1"` to skip Sentry init (synced with settings "disable crash reporting"). */
export const SENTRY_OPT_OUT_STORAGE_KEY = 'oclive.telemetry.sentryOptOut'

export function isSentryOptOut(): boolean {
  try {
    return globalThis.localStorage?.getItem(SENTRY_OPT_OUT_STORAGE_KEY) === '1'
  }
  catch {
    return false
  }
}

export function setSentryOptOut(optOut: boolean): void {
  try {
    if (optOut) {
      globalThis.localStorage?.setItem(SENTRY_OPT_OUT_STORAGE_KEY, '1')
    }
    else {
      globalThis.localStorage?.removeItem(SENTRY_OPT_OUT_STORAGE_KEY)
    }
  }
  catch {
    /* ignore quota / private mode */
  }
}

/** Load Sentry only when build-time DSN is set and user has not opted out. */
export function shouldLoadSentry(dsn: string | undefined): boolean {
  return typeof dsn === 'string' && dsn.length > 0 && !isSentryOptOut()
}

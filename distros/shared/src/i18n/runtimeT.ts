import { i18n } from './index'

/** Read current locale translations from non-component modules (Pinia, pure functions). */
export function rt(
  key: string,
  values?: Record<string, string | number | boolean>,
): string {
  return i18n.global.t(key, (values ?? {}) as Record<string, unknown>)
}

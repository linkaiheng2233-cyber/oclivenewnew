// @vitest-environment jsdom

import { createI18n } from 'vue-i18n'
import { describe, expect, it } from 'vitest'
import enUS from '../i18n/locales/en-US'
import zhCN from '../i18n/locales/zh-CN'

function collectPaths(value: unknown, path: string, out: string[]): void {
  if (typeof value === 'string') {
    out.push(path)
    return
  }
  if (value && typeof value === 'object' && !Array.isArray(value)) {
    for (const [k, v] of Object.entries(value as Record<string, unknown>)) {
      collectPaths(v, path ? `${path}.${k}` : k, out)
    }
  }
}

describe('i18n translate all keys', () => {
  for (const [locale, catalog] of [
    ['zh-CN', zhCN],
    ['en-US', enUS],
  ] as const) {
    it(`translates every ${locale} key without compile error`, () => {
      const i18n = createI18n({
        legacy: false,
        locale,
        fallbackLocale: 'zh-CN',
        messages: { 'zh-CN': zhCN, 'en-US': enUS },
      })
      const paths: string[] = []
      collectPaths(catalog, '', paths)
      const failures: string[] = []
      for (const p of paths) {
        try {
          i18n.global.t(p)
        }
        catch (e) {
          failures.push(`${p}: ${(e as Error).message}`)
        }
      }
      expect(failures, failures.join('\n')).toEqual([])
    })
  }
})

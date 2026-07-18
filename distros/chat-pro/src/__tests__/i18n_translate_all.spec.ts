// @vitest-environment jsdom

import enUS from '@oclive/shared/i18n/locales/en-US'
import zhCN from '@oclive/shared/i18n/locales/zh-CN'
import { describe, expect, it } from 'vitest'
import { createI18n } from 'vue-i18n'

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

function collectMessages(value: unknown, path: string, out: Array<[string, string]>): void {
  if (typeof value === 'string') {
    out.push([path, value])
    return
  }
  if (value && typeof value === 'object' && !Array.isArray(value)) {
    for (const [k, v] of Object.entries(value as Record<string, unknown>)) {
      collectMessages(v, path ? `${path}.${k}` : k, out)
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

    it(`keeps ${locale} messages free of HTML-like markup`, () => {
      const messages: Array<[string, string]> = []
      collectMessages(catalog, '', messages)
      const htmlMessages = messages
        .filter(([, message]) => /<\/?[a-z][^>]*>/i.test(message))
        .map(([path, message]) => `${path}: ${message}`)
      expect(htmlMessages, htmlMessages.join('\n')).toEqual([])
    })
  }
})

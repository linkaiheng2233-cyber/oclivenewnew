import { describe, expect, it } from 'vitest'
import enUS from '@oclive/shared/i18n/locales/en-US'
import zhCN from '@oclive/shared/i18n/locales/zh-CN'

/** Leaf key paths for nested message objects (arrays skipped). */
function flattenMessageKeys(obj: unknown, prefix = ''): Set<string> {
  const out = new Set<string>()
  if (obj === null || typeof obj !== 'object') {
    if (prefix)
      out.add(prefix)
    return out
  }
  if (Array.isArray(obj)) {
    return out
  }
  for (const k of Object.keys(obj as Record<string, unknown>)) {
    const path = prefix ? `${prefix}.${k}` : k
    const v = (obj as Record<string, unknown>)[k]
    if (v !== null && typeof v === 'object' && !Array.isArray(v)) {
      for (const p of flattenMessageKeys(v, path)) out.add(p)
    }
    else {
      out.add(path)
    }
  }
  return out
}

describe('i18n locale parity (zh-CN vs en-US)', () => {
  it('has the same key tree in both catalogs (no missing en-US leaves)', () => {
    const zhKeys = flattenMessageKeys(zhCN)
    const enKeys = flattenMessageKeys(enUS)
    const missingInEn = [...zhKeys].filter(k => !enKeys.has(k)).sort()
    expect(missingInEn, `Missing in en-US: ${missingInEn.join(', ')}`).toEqual([])
  })

  it('includes simplePluginManager.slots keys in both locales', () => {
    const slotKeys = [
      'simplePluginManager.slots.chat_toolbar',
      'simplePluginManager.slots.settings.panel',
      'simplePluginManager.slots.role.detail',
      'simplePluginManager.slots.sidebar',
      'simplePluginManager.slots.chat.header',
      'simplePluginManager.slots.settings.plugins',
      'simplePluginManager.slots.settings.advanced',
      'simplePluginManager.slots.overlay.floating',
      'simplePluginManager.slots.launcher.palette',
      'simplePluginManager.slots.debug.dock',
    ]
    const zhKeys = flattenMessageKeys(zhCN)
    const enKeys = flattenMessageKeys(enUS)
    for (const key of slotKeys) {
      expect(zhKeys.has(key), `zh-CN missing ${key}`).toBe(true)
      expect(enKeys.has(key), `en-US missing ${key}`).toBe(true)
    }
  })

  it('has the same key tree in both catalogs (no missing zh-CN leaves)', () => {
    const zhKeys = flattenMessageKeys(zhCN)
    const enKeys = flattenMessageKeys(enUS)
    const missingInZh = [...enKeys].filter(k => !zhKeys.has(k)).sort()
    expect(missingInZh, `Missing in zh-CN: ${missingInZh.join(', ')}`).toEqual([])
  })
})

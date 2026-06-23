// @vitest-environment jsdom

import type { DirectoryPluginCatalogEntry } from '@oclive/shared/api'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { defineComponent, h, nextTick } from 'vue'

import { useI18n } from 'vue-i18n'
import PmSlotRow from '@oclive/shared/components/PmSlotRow.vue'
import { i18n, LOCALE_PREF_KEY, setLocalePreference } from '@oclive/shared/i18n/index'
import enUS from '@oclive/shared/i18n/locales/en-US'
import zhCN from '@oclive/shared/i18n/locales/zh-CN'
import { usePluginStore } from '@oclive/shared/stores/pluginStore'
import ChatPanel from './fixtures/ChatPanel.vue'

const SettingsTitleProbe = defineComponent({
  name: 'SettingsTitleProbe',
  setup() {
    const { t } = useI18n()
    return () => h('div', { class: 'settings-probe' }, t('settings.title'))
  },
})

describe('vue-i18n infrastructure (smoke)', () => {
  beforeEach(() => {
    localStorage.clear()
  })

  afterEach(() => {
    setLocalePreference('system')
    localStorage.clear()
  })

  it('exposes a composition-mode i18n instance with zh-CN and en-US catalogs', () => {
    expect(i18n.mode).toBe('composition')
    const keys = Object.keys(i18n.global.messages.value)
    expect(keys).toEqual(expect.arrayContaining(['zh-CN', 'en-US']))
  })

  it('loads zh-CN and en-US locale modules (sanity)', () => {
    expect(zhCN.settings?.title).toBeTruthy()
    expect(enUS.settings?.title).toBeTruthy()
    expect(zhCN.chat?.demoTitle).toBeTruthy()
    expect(enUS.chat?.demoTitle).toBeTruthy()
  })

  it('persists locale preference in localStorage when using setLocalePreference', () => {
    setLocalePreference('en-US')
    expect(localStorage.getItem(LOCALE_PREF_KEY)).toBe('en-US')
    expect(i18n.global.locale.value).toBe('en-US')

    setLocalePreference('zh-CN')
    expect(localStorage.getItem(LOCALE_PREF_KEY)).toBe('zh-CN')
    expect(i18n.global.locale.value).toBe('zh-CN')
  })

  it('renders ChatPanel strings for zh-CN then en-US after locale switch', async () => {
    const pinia = createPinia()
    setLocalePreference('zh-CN')
    const w = mount(ChatPanel, {
      props: { message: '', busy: false },
      global: { plugins: [i18n, pinia] },
    })
    expect(w.text()).toContain(String(zhCN.chat.demoTitle))

    setLocalePreference('en-US')
    await nextTick()
    expect(w.text()).toContain(String(enUS.chat.demoTitle))
    w.unmount()
  })

  it('renders plugin manager slot row for zh-CN then en-US', async () => {
    const pinia = createPinia()
    setActivePinia(pinia)
    const store = usePluginStore()
    const entry: DirectoryPluginCatalogEntry = {
      id: 'test-plugin',
      version: '1',
      hasRpcProcess: false,
      isShell: false,
      uiSlotNames: ['chat_toolbar'],
      uiSlotVariants: [
        { slot: 'chat_toolbar', appearanceId: 'v1', label: 'Variant A' },
        { slot: 'chat_toolbar', appearanceId: 'v2', label: 'Variant B' },
      ],
      provides: [],
      dependencyStatus: 'ok',
      dependencyIssues: [],
    }
    store.$patch({ catalog: [entry] })

    setLocalePreference('zh-CN')
    const w = mount(PmSlotRow, {
      props: { pluginId: 'test-plugin', slotKey: 'chat_toolbar' },
      global: { plugins: [i18n, pinia] },
    })
    expect(w.text()).toContain(String(zhCN.pluginManager.pmSlot.hideSlot))

    setLocalePreference('en-US')
    await nextTick()
    expect(w.text()).toContain(String(enUS.pluginManager.pmSlot.hideSlot))
    w.unmount()
  })

  it('renders settings title under both locales', async () => {
    const pinia = createPinia()
    setLocalePreference('zh-CN')
    const w = mount(SettingsTitleProbe, {
      global: { plugins: [i18n, pinia] },
    })
    expect(w.text()).toContain(String(zhCN.settings.title))

    setLocalePreference('en-US')
    await nextTick()
    expect(w.text()).toContain(String(enUS.settings.title))
    w.unmount()
  })
})

// @vitest-environment jsdom

import { useAdultInteractionStore } from '@oclive/shared/stores/adultInteractionStore'
import { enableAutoUnmount, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { nextTick } from 'vue'
import { createI18n } from 'vue-i18n'
import settingsEn from '../../i18n/locales/fragments/settings.en'
import SettingsAdultTab from './SettingsAdultTab.vue'

enableAutoUnmount(afterEach)

const mocks = vi.hoisted(() => ({
  cancelAll: vi.fn(),
  cancelRole: vi.fn(),
  notifyCapacity: vi.fn(),
  roleStore: {
    currentRoleId: 'gentle-landlady',
    roleInfo: {
      name: 'Gentle Landlady',
      adultExtensionAvailable: true,
      adultExtensionError: null as string | null,
    },
    roles: [{
      id: 'gentle-landlady',
      name: 'Gentle Landlady',
      adultExtensionAvailable: true,
    }],
  },
}))

vi.mock('@oclive/shared/lib/adultBeatQueue', () => ({
  cancelAdultBeatQueuesForRole: mocks.cancelRole,
  cancelAllAdultBeatQueues: mocks.cancelAll,
  notifyAdultBeatQueueCapacityChanged: mocks.notifyCapacity,
}))

vi.mock('@oclive/shared/stores/roleStore', () => ({
  useRoleStore: () => mocks.roleStore,
}))

function mountTab() {
  const i18n = createI18n({
    legacy: false,
    locale: 'en',
    missingWarn: false,
    fallbackWarn: false,
    messages: {
      en: {
        ...settingsEn,
        app: {
          helpHintAria: 'Show explanation',
          helpHintCloseAria: 'Close explanation',
        },
      },
    },
  })
  return mount(SettingsAdultTab, {
    global: { plugins: [i18n] },
  })
}

function toggles(wrapper: ReturnType<typeof mountTab>) {
  return wrapper.findAll<HTMLInputElement>('.adult-switch input[type="checkbox"]')
}

describe('adult settings controlled toggles', () => {
  beforeEach(() => {
    localStorage.clear()
    vi.clearAllMocks()
    vi.spyOn(HTMLElement.prototype, 'getBoundingClientRect').mockReturnValue({
      x: 20,
      y: 20,
      width: 200,
      height: 40,
      top: 20,
      right: 220,
      bottom: 60,
      left: 20,
      toJSON: () => ({}),
    })
    setActivePinia(createPinia())
    mocks.roleStore.currentRoleId = 'gentle-landlady'
    mocks.roleStore.roleInfo = {
      name: 'Gentle Landlady',
      adultExtensionAvailable: true,
      adultExtensionError: null,
    }
    mocks.roleStore.roles = [{
      id: 'gentle-landlady',
      name: 'Gentle Landlady',
      adultExtensionAvailable: true,
      adultExtensionError: undefined,
    }]
  })

  afterEach(() => {
    document.body.innerHTML = ''
    vi.restoreAllMocks()
  })

  it('renders complete section copy instead of blank or unresolved UI fields', () => {
    const wrapper = mountTab()
    const sections = wrapper.findAll('.ui-section')

    expect(sections).toHaveLength(5)
    for (const section of sections) {
      expect(section.get('.ui-section__title').text().trim()).not.toBe('')
      expect(section.get('.ui-section__desc').text().trim()).not.toBe('')
    }
    expect(wrapper.text()).toContain('Gentle Landlady')
    expect(wrapper.text()).toContain('Currently effective:')
    expect(wrapper.text()).toContain('Current limit:')
    expect(wrapper.text()).not.toContain('settings.adult.')
  })

  it('opens the adult feature explanation from the question-mark hint on hover', async () => {
    const wrapper = mountTab()

    await wrapper.findAll('.help-hint')[0]!.trigger('pointerenter')
    await nextTick()

    expect(document.body.querySelector('.help-pop')?.textContent)
      .toContain('immediately ends all current adult interactions')
  })

  it('restores the global checkbox when immediate shutdown is declined', async () => {
    const store = useAdultInteractionStore()
    store.confirmAndEnableGlobal()
    const wrapper = mountTab()
    vi.spyOn(window, 'confirm').mockReturnValue(false)

    await toggles(wrapper)[0]!.setValue(false)

    expect(store.globalEnabled).toBe(true)
    expect(toggles(wrapper)[0]!.element.checked).toBe(true)
    expect(mocks.cancelAll).not.toHaveBeenCalled()
  })

  it('restores the queue checkbox when the background warning is declined', async () => {
    const store = useAdultInteractionStore()
    const wrapper = mountTab()
    vi.spyOn(window, 'confirm').mockReturnValue(false)

    await toggles(wrapper)[3]!.setValue(true)

    expect(store.backgroundQueueEnabled).toBe(false)
    expect(toggles(wrapper)[3]!.element.checked).toBe(false)
    expect(mocks.notifyCapacity).not.toHaveBeenCalled()
  })

  it('keeps an unconfirmed enable request off when the legal prompt is cancelled', async () => {
    const store = useAdultInteractionStore()
    const wrapper = mountTab()

    await toggles(wrapper)[0]!.setValue(true)

    expect(store.globalEnabled).toBe(false)
    expect(toggles(wrapper)[0]!.element.checked).toBe(false)
    expect(wrapper.find('[role="dialog"]').exists()).toBe(true)

    await wrapper.get('[role="dialog"] .ui-btn--secondary').trigger('click')
    expect(store.globalEnabled).toBe(false)
    expect(toggles(wrapper)[0]!.element.checked).toBe(false)
  })

  it('confirms adulthood once and enables the requested global and role gates', async () => {
    const store = useAdultInteractionStore()
    const wrapper = mountTab()

    await toggles(wrapper)[0]!.setValue(true)
    await wrapper.get('[role="dialog"] .ui-btn--primary').trigger('click')
    await nextTick()

    expect(store.confirmedAdult).toBe(true)
    expect(store.globalEnabled).toBe(true)
    expect(toggles(wrapper)[0]!.element.checked).toBe(true)

    await toggles(wrapper)[1]!.setValue(true)
    expect(store.roleIsEnabled('gentle-landlady')).toBe(true)
    expect(toggles(wrapper)[1]!.element.checked).toBe(true)
  })

  it('persists the pacing override when its switch changes', async () => {
    const store = useAdultInteractionStore()
    const wrapper = mountTab()

    await toggles(wrapper)[2]!.setValue(true)

    expect(store.pacingOverrideEnabled).toBe(true)
    expect(JSON.parse(
      localStorage.getItem('oclive-chat-pro-adult-settings-v1') ?? '{}',
    )).toMatchObject({
      pacingOverrideEnabled: true,
    })
  })

  it('rejects out-of-range pacing and queue inputs without changing settings', async () => {
    const store = useAdultInteractionStore()
    const wrapper = mountTab()
    const inputs = wrapper.findAll<HTMLInputElement>('.adult-number-control input')

    await inputs[0]!.setValue(499)
    await wrapper.findAll('button').find(button => button.text().includes('Save pacing'))!.trigger('click')
    expect(store.pacingIntervalMs).toBe(4_000)
    expect(wrapper.text()).toContain('500 through 60000')

    await inputs[1]!.setValue(9)
    await wrapper.findAll('button').find(button => button.text().includes('Save cache'))!.trigger('click')
    expect(store.backgroundQueueCap).toBe(2)
    expect(wrapper.text()).toContain('1 through 8')
  })

  it('shows a prominent error while keeping an invalid-extension role available ordinarily', () => {
    mocks.roleStore.roles = [{
      id: 'broken-role',
      name: 'Broken Role',
      adultExtensionAvailable: false,
      adultExtensionError: 'adult_extension.json parse failed at line 1',
    }]
    mocks.roleStore.currentRoleId = ''
    const wrapper = mountTab()

    expect(wrapper.get('[role="alert"]').text()).toContain('Some adult extensions were disabled')
    expect(wrapper.get('[role="alert"]').text()).toContain('Broken Role')
    expect(wrapper.text()).toContain('No manageable roles yet')
  })

  it('cancels active generation before resetting confirmation and R18 settings', async () => {
    const store = useAdultInteractionStore()
    store.confirmAndEnableGlobal()
    store.setRoleEnabled('gentle-landlady', true)
    store.setBackgroundQueue(true, 4, true)
    const wrapper = mountTab()
    vi.spyOn(window, 'confirm').mockReturnValue(true)

    await wrapper.get('.adult-reset-button').trigger('click')
    await nextTick()

    expect(mocks.cancelAll).toHaveBeenCalledTimes(1)
    expect(store.confirmedAdult).toBe(false)
    expect(store.globalEnabled).toBe(false)
    expect(store.roleEnabled).toEqual({})
    expect(store.backgroundQueueCap).toBe(2)
  })
})

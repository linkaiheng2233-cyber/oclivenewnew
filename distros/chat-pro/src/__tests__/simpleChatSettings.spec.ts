// @vitest-environment jsdom
import { useUiStore } from '@oclive/shared/stores/uiStore'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent } from 'vue'
import SettingsView from '../views/SettingsView.vue'

const mocks = vi.hoisted(() => ({
  voiceMounted: vi.fn(),
  pluginsMounted: vi.fn(),
  adultMounted: vi.fn(),
  roleStore: { interactionImmersive: true, roleInfo: { interactionMode: 'immersive' } },
  adultStore: { globalEnabled: false },
  pluginStore: {
    bootstrapEpoch: 0,
    bootstrapUiSlots: [
      { slot: 'settings.panel', pluginId: 'com.oclive.voice.asr' },
      { slot: 'settings.panel', pluginId: 'fixture-plugin' },
    ],
  },
}))
vi.mock('@oclive/shared/stores/roleStore', () => ({ useRoleStore: () => mocks.roleStore }))
vi.mock('@oclive/shared/stores/adultInteractionStore', () => ({ useAdultInteractionStore: () => mocks.adultStore }))
vi.mock('@oclive/shared/stores/pluginStore', () => ({
  usePluginStore: () => mocks.pluginStore,
  SLOT_SETTINGS_PANEL: 'settings.panel',
  isPureChatPlatformPlugin: () => true,
}))
vi.mock('vue-i18n', () => ({ useI18n: () => ({ t: (key: string) => key }) }))

const wrappers: Array<ReturnType<typeof mount>> = []
function render(focusTab: 'general' | 'voice' | 'plugins' | 'storage' | 'adult' = 'general', basicWorkspace = false) {
  const wrapper = mount(SettingsView, {
    props: { visible: true, embedded: true, focusTab, basicWorkspace },
    global: {
      stubs: {
        SettingsGeneralTab: { template: '<div data-core-settings />' },
        ChatStorageSettingsPanel: { template: '<div data-storage />' },
        SettingsVoiceTab: defineComponent({ setup: mocks.voiceMounted, template: '<div data-voice />' }),
        SettingsPluginsTab: defineComponent({ setup: mocks.pluginsMounted, template: '<div data-plugins />' }),
        SettingsAdultTab: defineComponent({ setup: mocks.adultMounted, template: '<div data-adult />' }),
      },
    },
  })
  wrappers.push(wrapper)
  return wrapper
}

describe('simple chat settings', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    mocks.adultStore.globalEnabled = false
  })
  afterEach(() => wrappers.splice(0).forEach(wrapper => wrapper.unmount()))

  it('defaults advanced features off without changing the saved scene', () => {
    const ui = useUiStore()
    ui.setScene('saved-scene')
    expect(ui.advancedFeaturesEnabled).toBe(false)
    ui.setAdvancedFeaturesEnabled(true)
    ui.setAdvancedFeaturesEnabled(false)
    expect(ui.sceneId).toBe('saved-scene')
  })

  it('does not mount or warm optional tabs when ordinary settings opens', async () => {
    const wrapper = render()
    await flushPromises()
    expect(wrapper.find('[data-core-settings]').exists()).toBe(true)
    expect(wrapper.text()).toContain('settings.tabStorage')
    expect(wrapper.text()).not.toContain('settings.tabVoice')
    expect(wrapper.text()).not.toContain('settings.tabPlugins')
    expect(mocks.voiceMounted).not.toHaveBeenCalled()
    expect(mocks.pluginsMounted).not.toHaveBeenCalled()
    expect(mocks.adultMounted).not.toHaveBeenCalled()
  })

  it('rejects an advanced focusTab while advanced features are off', async () => {
    const wrapper = render('voice')
    await flushPromises()
    expect(wrapper.find('[data-core-settings]').exists()).toBe(true)
    expect(mocks.voiceMounted).not.toHaveBeenCalled()
  })

  it('loads voice only on its selected tab and unmounts it when advanced closes', async () => {
    const ui = useUiStore()
    ui.setAdvancedFeaturesEnabled(true)
    const wrapper = render()
    await flushPromises()
    expect(mocks.voiceMounted).not.toHaveBeenCalled()
    await wrapper.setProps({ focusTab: 'voice' })
    await flushPromises()
    expect(wrapper.find('[data-voice]').exists()).toBe(true)
    ui.setAdvancedFeaturesEnabled(false)
    await flushPromises()
    expect(wrapper.find('[data-voice]').exists()).toBe(false)
    expect(wrapper.find('[data-core-settings]').exists()).toBe(true)
  })

  it('keeps deferred feature hosts unavailable in the standard role workspace even with old preferences', async () => {
    useUiStore().setAdvancedFeaturesEnabled(true)
    const wrapper = render('voice', true)
    await flushPromises()
    expect(wrapper.find('[data-core-settings]').exists()).toBe(true)
    expect(wrapper.text()).not.toContain('settings.tabVoice')
    expect(wrapper.text()).not.toContain('settings.tabPlugins')
    expect(wrapper.text()).not.toContain('settings.tabAdult')
    expect(mocks.voiceMounted).not.toHaveBeenCalled()
    expect(mocks.pluginsMounted).not.toHaveBeenCalled()
  })

  it('keeps the exit and management settings reachable for an already-enabled adult feature', async () => {
    mocks.adultStore.globalEnabled = true
    const wrapper = render('adult')
    await flushPromises()
    expect(wrapper.find('[data-adult]').exists()).toBe(true)
    expect(mocks.voiceMounted).not.toHaveBeenCalled()
  })

  it('keeps everyday sections separate and passes only the selected page to general settings', async () => {
    const wrapper = render('general', true)
    await flushPromises()
    expect(wrapper.findAll('[data-settings-page]')).toHaveLength(6)
    expect(wrapper.find('[data-settings-page="appearance"]').attributes('aria-current')).toBe('page')
    await wrapper.find('[data-settings-page="connection"]').trigger('click')
    expect(wrapper.find('[data-core-settings]').attributes('page')).toBe('connection')
    expect(wrapper.findAll('[aria-current="page"]')).toHaveLength(1)
    await wrapper.find('[data-settings-page="storage"]').trigger('click')
    await flushPromises()
    expect(wrapper.find('[data-storage]').exists()).toBe(true)
    expect(wrapper.find('[data-core-settings]').exists()).toBe(false)
    await wrapper.find('[data-settings-page="roles"]').trigger('click')
    await flushPromises()
    expect(wrapper.find('[data-core-settings]').attributes('page')).toBe('roles')
  })
})

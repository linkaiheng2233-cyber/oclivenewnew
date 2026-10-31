// @vitest-environment jsdom
import { i18n } from '@oclive/shared/i18n'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia } from 'pinia'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import SettingsGeneralTab from './SettingsGeneralTab.vue'

const mocks = vi.hoisted(() => ({
  loadFallback: vi.fn(),
  saveFallback: vi.fn(),
  diagnostics: vi.fn(),
  repair: vi.fn(),
  openPath: vi.fn(),
  persist: vi.fn(),
  plugin: { pluginState: { force_iframe_mode: false }, bootstrapEpoch: 0 },
}))
vi.mock('@oclive/shared/api', () => ({
  getRemoteFallbackAppSettings: mocks.loadFallback,
  setRemoteFallbackToBuiltin: mocks.saveFallback,
  runEnvironmentDiagnostics: mocks.diagnostics,
  runEnvironmentRepair: mocks.repair,
  openPathInFileManager: mocks.openPath,
}))
vi.mock('@oclive/shared/stores/pluginStore', () => ({
  usePluginStore: () => ({ ...mocks.plugin, persist: mocks.persist }),
  SLOT_SETTINGS_ADVANCED: 'settings.advanced',
}))
vi.mock('@oclive/shared/stores/roleStore', () => ({ useRoleStore: () => ({ roleInfo: { interactionMode: 'pure_chat' } }) }))
vi.mock('@oclive/shared/composables/useAppToast', () => ({ useAppToast: () => ({ showToast: vi.fn() }) }))
vi.mock('@oclive/shared/composables/useDistroUxProfile', async () => ({ useDistroUxProfile: () => ({ allowModeSwitch: true, ensureDistroUxProfileLoaded: vi.fn() }) }))
vi.mock('@oclive/shared/composables/useInteractionModeSettings', () => ({ useInteractionModeSettings: () => ({ onInteractionModeSelect: vi.fn() }) }))
vi.mock('@oclive/shared/composables/useUserIdentityState', () => ({ useUserIdentityState: () => ({ hasCatalog: true }) }))
vi.mock('@oclive/shared/composables/useOcliveAppearance', () => ({ useOcliveAppearance: () => ({ themeCycleLabel: 'Theme', cycleTheme: vi.fn(), bumpScale: vi.fn(), scaleLabel: '100%' }) }))
vi.mock('@oclive/shared/composables/useEasterEggSkin', () => ({ useEasterEggSkin: () => ({ skinUnlocked: false, win98Enabled: false, toggleWin98: vi.fn() }) }))
vi.mock('@oclive/shared/utils/vueComponentSecurity', () => ({ isUnsafeInlinePluginVueEnabled: () => true }))
// The panel is deliberately stubbed in these settings tests. Mock its async
// loader too, so it cannot keep importing the chat graph after unmount/teardown.
vi.mock('./KernelConnectionSettingsPanel.vue', () => ({
  default: { name: 'KernelConnectionSettingsPanel', template: '<div />' },
}))

const wrappers: Array<ReturnType<typeof mount>> = []
function render(page = 'connection') {
  const wrapper = mount(SettingsGeneralTab, {
    props: { page, visible: true, embedded: false, generalSubTab: 'simple', basicWorkspace: true },
    global: { plugins: [createPinia(), i18n], stubs: {
      KeybindingsSettingsSection: true,
      RoleIdentityControls: true,
      RolePackBar: true,
      HelpHint: true,
      PluginSlotEmbed: true,
      ReplyPostProcessorStatus: true,
      KernelConnectionSettingsPanel: true,
    } },
  })
  wrappers.push(wrapper)
  return wrapper
}
function deferred() {
  let resolve!: () => void
  let reject!: (reason: unknown) => void
  const promise = new Promise<void>((yes, no) => {
    resolve = yes
    reject = no
  })
  return { promise, resolve, reject }
}

describe('settings save and recovery paths', () => {
  beforeEach(() => {
    vi.resetAllMocks()
    localStorage.clear()
    i18n.global.locale.value = 'en-US'
    mocks.loadFallback.mockResolvedValue({ remoteFallbackEnvOverrideActive: false, remoteFallbackToBuiltin: '1' })
    mocks.saveFallback.mockResolvedValue(undefined)
  })
  afterEach(() => {
    wrappers.splice(0).forEach(wrapper => wrapper.unmount())
    vi.restoreAllMocks()
  })

  it('disables an unread fallback setting, then makes a failed read explicitly retryable', async () => {
    mocks.loadFallback.mockRejectedValueOnce('{"code":"IO_ERROR","message":"cannot read preferences api_key=secret-settings-test"}')
    const wrapper = render()
    await flushPromises()
    expect(wrapper.get('input[role=switch]').attributes('disabled')).toBeDefined()
    expect(wrapper.get('[role=alert]').text()).toContain('could not be loaded')
    expect(wrapper.get('[role=alert]').text()).not.toContain('secret-settings-test')
    await wrapper.get('[role=alert] button.ui-btn').trigger('click')
    await flushPromises()
    expect(wrapper.find('[role=alert]').exists()).toBe(false)
    expect(wrapper.get('input[role=switch]').attributes('disabled')).toBeUndefined()
  })

  it('prevents overlapping saves, rolls a failed save back, and retries the requested value', async () => {
    const saving = deferred()
    mocks.saveFallback.mockReturnValueOnce(saving.promise)
    const wrapper = render()
    await flushPromises()
    await wrapper.get('input[role=switch]').setValue(false)
    expect(wrapper.get('input[role=switch]').attributes('disabled')).toBeDefined()
    expect(wrapper.text()).toContain('Saving')
    expect(mocks.saveFallback).toHaveBeenCalledTimes(1)
    await wrapper.setProps({ page: 'appearance' })
    await wrapper.setProps({ page: 'connection' })
    expect(mocks.loadFallback).toHaveBeenCalledTimes(1)
    saving.reject('{"code":"IO_ERROR","message":"preferences not writable"}')
    await flushPromises()
    expect((wrapper.get('input[role=switch]').element as HTMLInputElement).checked).toBe(true)
    expect(wrapper.get('[role=alert]').text()).toContain('previous choice was restored')
    await wrapper.get('[role=alert] button.ui-btn').trigger('click')
    await flushPromises()
    expect(mocks.saveFallback.mock.calls).toEqual([[false], [false]])
    expect((wrapper.get('input[role=switch]').element as HTMLInputElement).checked).toBe(false)
  })

  it('honors environment-locked preferences and never writes them', async () => {
    mocks.loadFallback.mockResolvedValue({ remoteFallbackEnvOverrideActive: true, remoteFallbackToBuiltin: '0' })
    const wrapper = render()
    await flushPromises()
    expect(wrapper.get('input[role=switch]').attributes('disabled')).toBeDefined()
    expect(mocks.saveFallback).not.toHaveBeenCalled()
  })

  it('contains an environment check failure and recovers without closing settings', async () => {
    mocks.diagnostics.mockRejectedValueOnce('{"code":"IO_ERROR","message":"diagnostic directory denied"}')
    mocks.diagnostics.mockResolvedValueOnce({ ollamaReachable: true, rolesDirExists: true, rolesDirReadable: true, appDataWritable: true })
    const wrapper = render()
    await flushPromises()
    const check = wrapper.findAll('button').find(button => button.text() === 'Run check')!
    await check.trigger('click')
    await flushPromises()
    expect(wrapper.get('[role=alert]').text()).toContain('environment check could not finish')
    await wrapper.get('[role=alert] button.ui-btn').trigger('click')
    await flushPromises()
    expect(wrapper.find('[role=alert]').exists()).toBe(false)
    expect(wrapper.find('.sv-env-results').exists()).toBe(true)
  })

  it('keeps late failures on their own page when users navigate away', async () => {
    const loading = deferred()
    mocks.loadFallback.mockReturnValueOnce(loading.promise)
    const wrapper = render()
    await wrapper.setProps({ page: 'appearance' })
    loading.reject(new Error('late failure'))
    await flushPromises()
    expect(wrapper.find('[role=alert]').exists()).toBe(false)
  })

  it('reports local preference storage failure and restores the visible switch', async () => {
    const original = Storage.prototype.setItem
    vi.spyOn(Storage.prototype, 'setItem').mockImplementation(function (key, value) {
      if (key === 'oclive.chat.streamEnabled')
        throw new Error('Preference storage is full')
      original.call(this, key, value)
    })
    const wrapper = render('chat')
    await wrapper.get('input[role=switch]').setValue(false)
    await flushPromises()
    expect((wrapper.get('input[role=switch]').element as HTMLInputElement).checked).toBe(true)
    expect(wrapper.get('[role=alert]').text()).toContain('previous choice was restored')
    vi.restoreAllMocks()
    await wrapper.get('[role=alert] button.ui-btn').trigger('click')
    await flushPromises()
    expect(localStorage.getItem('oclive.chat.streamEnabled')).toBe('false')
  })
})

// @vitest-environment jsdom
import type { LlmUserSettings } from '@oclive/shared/api/llmSettings'
import { useUiStore } from '@oclive/shared/stores/uiStore'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia } from 'pinia'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createI18n } from 'vue-i18n'
import ModelSetupGuide from '../components/ModelSetupGuide.vue'

const api = vi.hoisted(() => ({ settings: vi.fn() }))
vi.mock('@oclive/shared/api/llmSettings', () => ({ getLlmUserSettings: api.settings }))
const wrappers: Array<ReturnType<typeof mount>> = []
function start(roleId = '', ready = false) {
  const pinia = createPinia()
  const ui = useUiStore(pinia)
  const wrapper = mount(ModelSetupGuide, {
    props: { roleId, ready },
    global: { plugins: [pinia, createI18n({ legacy: false, locale: 'en', missingWarn: false, fallbackWarn: false, messages: { en: {} } })] },
  })
  wrappers.push(wrapper)
  return { wrapper, ui }
}
const emptySettings = {
  provider: 'local',
  localModelPath: '',
  sessionOllamaModel: null,
  effectiveModel: 'pack-default-is-not-installed',
  performanceReady: false,
  remoteUrl: '',
  remoteModel: '',
} as LlmUserSettings

describe('first-run model choice', () => {
  beforeEach(() => {
    vi.resetAllMocks()
    api.settings.mockResolvedValue(emptySettings)
  })
  afterEach(() => wrappers.splice(0).forEach(wrapper => wrapper.unmount()))

  it('keeps the role library available without activating a role or contacting a model', async () => {
    const { wrapper } = start()
    await flushPromises()
    expect(wrapper.find('.model-setup').exists()).toBe(true)
    expect(api.settings).not.toHaveBeenCalled()
    expect(wrapper.emitted('configure')).toBeUndefined()
  })

  it('remembers a cloud choice until the selected character is ready', async () => {
    const { wrapper } = start()
    await wrapper.get('.setup-options button:last-child').trigger('click')
    expect(wrapper.text()).toContain('workspace.setupSelectRole')
    await wrapper.setProps({ roleId: 'mumu', ready: false })
    expect(wrapper.emitted('configure')).toBeUndefined()
    await wrapper.setProps({ ready: true })
    expect(wrapper.emitted('configure')).toEqual([['cloud']])
  })

  it('opens the requested local route without rewriting existing settings', async () => {
    const { wrapper } = start('mumu', true)
    await flushPromises()
    await wrapper.get('.setup-options button:first-child').trigger('click')
    expect(wrapper.emitted('configure')).toEqual([['local']])
  })

  it('does not treat a pack default as proof that a local model is installed', async () => {
    const { wrapper } = start('mumu', true)
    await flushPromises()
    expect(wrapper.find('.model-setup').exists()).toBe(true)
  })

  it.each([
    { ...emptySettings, provider: 'cloud', remoteUrl: 'https://api.example.test/v1', remoteModel: 'saved-model', remoteTokenConfigured: true },
    { ...emptySettings, localModelPath: 'E:/models/existing.gguf' },
    { ...emptySettings, sessionOllamaModel: 'existing:7b' },
  ])('respects an existing connection on upgrade without probing inference', async (settings) => {
    api.settings.mockResolvedValue(settings)
    const { wrapper } = start('mumu', true)
    await flushPromises()
    expect(wrapper.find('.model-setup').exists()).toBe(false)
    expect(wrapper.emitted('configure')).toBeUndefined()
  })

  it('keeps setup available for an incomplete cloud connection without a key', async () => {
    api.settings.mockResolvedValue({ ...emptySettings, provider: 'cloud', remoteUrl: 'https://api.example.test/v1', remoteModel: 'saved-model' })
    const { wrapper } = start('mumu', true)
    await flushPromises()
    expect(wrapper.find('.model-setup').exists()).toBe(true)
  })

  it('can defer setup and never opens a queued choice after deferring', async () => {
    const { wrapper, ui } = start()
    await wrapper.get('.setup-options button:first-child').trigger('click')
    await wrapper.get('.setup-later').trigger('click')
    await wrapper.setProps({ roleId: 'mumu', ready: true })
    expect(ui.modelSetupDismissed).toBe(true)
    expect(wrapper.find('.model-setup').exists()).toBe(false)
    expect(wrapper.emitted('configure')).toBeUndefined()
  })

  it('ignores late settings from a previous role and supports retry after a read failure', async () => {
    let finish!: (value: LlmUserSettings) => void
    api.settings.mockReturnValueOnce(new Promise<LlmUserSettings>((resolve) => {
      finish = resolve
    }))
    const { wrapper } = start('old-role', true)
    api.settings.mockRejectedValueOnce(new Error('[NETWORK_ERROR] settings unavailable'))
    await wrapper.setProps({ roleId: 'new-role' })
    await flushPromises()
    finish({ ...emptySettings, sessionOllamaModel: 'old-model' })
    await flushPromises()
    expect(wrapper.find('.model-setup').exists()).toBe(true)
    expect(wrapper.text()).toContain('workspace.setupCheckFailed')
    const retry = wrapper.findAll('button').find(button => button.text() === 'workspace.retry')!
    await retry.trigger('click')
    await flushPromises()
    expect(api.settings).toHaveBeenLastCalledWith('new-role')
    expect(wrapper.text()).not.toContain('workspace.setupCheckFailed')
  })
})

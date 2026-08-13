// @vitest-environment jsdom

import type { LlmUserSettings } from '@oclive/shared/api/llmSettings'
import { mount } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { nextTick } from 'vue'
import { createI18n } from 'vue-i18n'
import ModelManagerBody from './ModelManagerBody.vue'

const mocks = vi.hoisted(() => ({
  getGlobal: vi.fn(),
  getSettings: vi.fn(),
  listCloud: vi.fn(),
  listOllama: vi.fn(),
  saveSettings: vi.fn(),
  setGlobal: vi.fn(),
  showToast: vi.fn(),
}))

vi.mock('@oclive/shared/api/llmSettings', () => ({
  activateLocalLoraAdapter: vi.fn(),
  deleteLocalLoraAdapter: vi.fn(),
  getGlobalOllamaModel: mocks.getGlobal,
  getLlmUserSettings: mocks.getSettings,
  importGgufToOllama: vi.fn(),
  importLocalLoraAdapter: vi.fn(),
  listCloudModels: mocks.listCloud,
  listOllamaModels: mocks.listOllama,
  openPathInFileManager: vi.fn(),
  probeCloudLlm: vi.fn(),
  saveLlmUserSettings: mocks.saveSettings,
  scanLocalModelFiles: vi.fn(),
  setGlobalOllamaModel: mocks.setGlobal,
}))

vi.mock('@oclive/shared/composables/useAppToast', () => ({
  useAppToast: () => ({ showToast: mocks.showToast }),
}))

vi.mock('@oclive/shared/stores/roleStore', async () => {
  const { reactive } = await import('vue')
  const store = reactive({
    currentRoleId: 'role-a',
    roleInfo: { effectiveOllamaModel: '' },
    applyRoleInfo: vi.fn(),
    refreshRoleInfo: vi.fn(),
  })
  return { useRoleStore: () => store }
})

vi.mock('@tauri-apps/plugin-dialog', () => ({
  confirm: vi.fn(),
  open: vi.fn(),
}))

function deferred<T>() {
  let resolve!: (value: T) => void
  const promise = new Promise<T>((resolvePromise) => {
    resolve = resolvePromise
  })
  return { promise, resolve }
}

function settingsFor(
  ollamaBaseUrl: string,
  provider: 'local' | 'cloud' = 'local',
): LlmUserSettings {
  return {
    provider,
    cloudVendor: '',
    cloudApiStyle: 'openai',
    ollamaBaseUrl,
    ollamaReachable: true,
    ollamaDetail: '',
    localModelsDir: '',
    localModelFiles: [],
    localModelPath: '',
    localLoraAdapters: [],
    activeLocalLoraAdapterId: null,
    localRuntimeMode: 'ollama',
    performanceEndpoint: '',
    performanceRuntimeAvailable: false,
    performanceModelConfigured: false,
    performanceReady: false,
    performanceActiveBackend: 'ollama',
    performanceDetail: '',
    packOllamaModel: null,
    sessionOllamaModel: null,
    effectiveModel: '',
    remoteUrl: '',
    remoteTokenConfigured: false,
    remoteModel: '',
    remoteUrlEnvActive: false,
    remoteTokenEnvActive: false,
  }
}

async function flushMicrotasks(rounds = 12): Promise<void> {
  for (let index = 0; index < rounds; index += 1)
    await Promise.resolve()
  await nextTick()
}

function mountManager(useRealSharedComponents = false) {
  const i18n = createI18n({
    legacy: false,
    locale: 'en',
    missingWarn: false,
    fallbackWarn: false,
    messages: { en: {} },
  })
  return mount(ModelManagerBody, {
    global: {
      plugins: [i18n],
      stubs: useRealSharedComponents
        ? {}
        : {
            HelpHint: true,
            UiButton: true,
          },
    },
  })
}

describe('model manager async ownership', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mocks.getGlobal.mockResolvedValue({ model: '' })
    mocks.listCloud.mockResolvedValue([])
    mocks.listOllama.mockResolvedValue([])
    mocks.saveSettings.mockResolvedValue({})
    mocks.setGlobal.mockResolvedValue({ model: 'new-model' })
  })

  afterEach(async () => {
    const { useRoleStore } = await import('@oclive/shared/stores/roleStore')
    useRoleStore().currentRoleId = 'role-a'
    document.body.innerHTML = ''
  })

  it('keeps the latest role settings when an older request finishes last', async () => {
    const roleA = deferred<LlmUserSettings>()
    const roleB = deferred<LlmUserSettings>()
    mocks.getSettings.mockImplementation((roleId: string) =>
      roleId === 'role-a' ? roleA.promise : roleB.promise,
    )
    const wrapper = mountManager()
    await flushMicrotasks()
    expect(mocks.getSettings).toHaveBeenCalledWith('role-a')

    const { useRoleStore } = await import('@oclive/shared/stores/roleStore')
    useRoleStore().currentRoleId = 'role-b'
    await flushMicrotasks()
    expect(mocks.getSettings).toHaveBeenCalledWith('role-b')

    roleB.resolve(settingsFor('http://role-b.test'))
    await flushMicrotasks()
    roleA.resolve(settingsFor('http://stale-role-a.test'))
    await flushMicrotasks()

    const baseUrl = wrapper.get('input[type="url"].mm-input')
    expect((baseUrl.element as HTMLInputElement).value).toBe('http://role-b.test')
    expect(mocks.listOllama).toHaveBeenCalledWith('http://role-b.test')
    expect(mocks.listOllama).not.toHaveBeenCalledWith('http://stale-role-a.test')

    wrapper.unmount()
  })

  it('discards an older provider-list response after a newer refresh', async () => {
    mocks.getSettings.mockResolvedValue(settingsFor('http://initial.test'))
    mocks.listOllama.mockResolvedValueOnce([])
    const wrapper = mountManager()
    await flushMicrotasks()

    const staleModels = deferred<string[]>()
    mocks.listOllama
      .mockReturnValueOnce(staleModels.promise)
      .mockResolvedValueOnce(['new-model'])
    const baseUrl = wrapper.get('input[type="url"].mm-input')
    const refresh = wrapper.get(
      '.mm-global-default .mm-row-actions button:last-child',
    )

    await baseUrl.setValue('http://old.test')
    await refresh.trigger('click')
    await flushMicrotasks()
    await baseUrl.setValue('http://new.test')

    staleModels.resolve(['stale-model'])
    await flushMicrotasks()
    await refresh.trigger('click')
    await flushMicrotasks()

    const optionValues = wrapper
      .findAll<HTMLOptionElement>('option')
      .map(option => option.attributes('value'))
    expect(optionValues).toContain('new-model')
    expect(optionValues).not.toContain('stale-model')

    wrapper.unmount()
  })

  it('keeps role settings usable when the global default request fails', async () => {
    mocks.getGlobal.mockRejectedValueOnce(new Error('global unavailable'))
    mocks.getSettings.mockResolvedValue(settingsFor('http://role-settings.test'))
    const wrapper = mountManager()
    await flushMicrotasks()

    expect((wrapper.get('input[type="url"].mm-input').element as HTMLInputElement).value)
      .toBe('http://role-settings.test')
    expect(mocks.showToast).toHaveBeenCalledWith('error', 'global unavailable')

    wrapper.unmount()
  })

  it('refreshes role info through the real store action after global save', async () => {
    mocks.getGlobal.mockResolvedValue({ model: 'old-model' })
    mocks.getSettings.mockResolvedValue(settingsFor('http://role.test'))
    mocks.listOllama.mockResolvedValue(['old-model', 'new-model'])
    const wrapper = mountManager()
    await flushMicrotasks()

    await wrapper.get('.mm-global-default select').setValue('new-model')
    await wrapper.get('.mm-global-default .mm-btn-primary').trigger('click')
    await flushMicrotasks()

    const { useRoleStore } = await import('@oclive/shared/stores/roleStore')
    expect(mocks.setGlobal).toHaveBeenCalledWith('new-model', 'role-a')
    expect(useRoleStore().refreshRoleInfo).toHaveBeenCalledTimes(1)

    wrapper.unmount()
  })

  it('does not apply an old role save response after switching roles', async () => {
    const initial = settingsFor('http://role-a.test')
    initial.sessionOllamaModel = 'model-a'
    initial.effectiveModel = 'model-a'
    mocks.getSettings.mockResolvedValue(initial)
    mocks.listOllama.mockResolvedValue(['model-a'])
    const saved = deferred<Record<string, unknown>>()
    mocks.saveSettings.mockReturnValueOnce(saved.promise)
    const wrapper = mountManager()
    await flushMicrotasks()

    await wrapper.get('.mm-footer ui-button-stub').trigger('click')
    await flushMicrotasks()
    const { useRoleStore } = await import('@oclive/shared/stores/roleStore')
    useRoleStore().currentRoleId = 'role-b'
    await flushMicrotasks()
    saved.resolve({})
    await flushMicrotasks()

    expect(useRoleStore().applyRoleInfo).not.toHaveBeenCalled()

    wrapper.unmount()
  })

  it('renders the real shared controls used by the production template', async () => {
    const settings = settingsFor('http://role.test')
    settings.localRuntimeMode = 'performance'
    mocks.getSettings.mockResolvedValue(settings)
    const wrapper = mountManager(true)
    await flushMicrotasks()

    expect(wrapper.find('.mm-lora-route .help-hint').exists()).toBe(true)
    expect(wrapper.findAll('.mm-footer button.ui-btn')).toHaveLength(2)

    wrapper.unmount()
  })
})

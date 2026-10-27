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
  scan: vi.fn(),
  probe: vi.fn(),
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
  probeCloudLlm: mocks.probe,
  saveLlmUserSettings: mocks.saveSettings,
  scanLocalModelFiles: mocks.scan,
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
  let reject!: (reason: unknown) => void
  const promise = new Promise<T>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise
    reject = rejectPromise
  })
  return { promise, resolve, reject }
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

function cloudSettings(): LlmUserSettings {
  return {
    ...settingsFor('http://local.test', 'cloud'),
    remoteUrl: 'https://saved.test/v1',
    remoteModel: 'saved-model',
    remoteTokenConfigured: true,
  }
}

async function flushMicrotasks(rounds = 12): Promise<void> {
  for (let index = 0; index < rounds; index += 1)
    await Promise.resolve()
  await nextTick()
}

const wrappers: Array<ReturnType<typeof mount>> = []
function mountManager(useRealSharedComponents = false, initialProvider?: 'local' | 'cloud') {
  const i18n = createI18n({
    legacy: false,
    locale: 'en',
    missingWarn: false,
    fallbackWarn: false,
    messages: { en: {} },
  })
  const wrapper = mount(ModelManagerBody, {
    props: { initialProvider },
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
  wrappers.push(wrapper)
  return wrapper
}

describe('model manager async ownership', () => {
  beforeEach(() => {
    vi.resetAllMocks()
    mocks.getGlobal.mockResolvedValue({ model: '' })
    mocks.listCloud.mockResolvedValue([])
    mocks.listOllama.mockResolvedValue([])
    mocks.saveSettings.mockResolvedValue({})
    mocks.setGlobal.mockResolvedValue({ model: 'new-model' })
    mocks.probe.mockResolvedValue('ok')
  })

  afterEach(async () => {
    wrappers.splice(0).forEach((wrapper) => {
      if (wrapper.exists())
        wrapper.unmount()
    })
    const { useRoleStore } = await import('@oclive/shared/stores/roleStore')
    useRoleStore().currentRoleId = 'role-a'
    document.body.innerHTML = ''
  })

  it('honors the first-run cloud choice instead of the default local provider', async () => {
    mocks.getSettings.mockResolvedValue(settingsFor('http://local.test'))
    const wrapper = mountManager(false, 'cloud')
    await flushMicrotasks()
    expect(wrapper.findAll('[role="tab"]')[1].attributes('aria-selected')).toBe('true')
    expect(wrapper.find('.mm-global-default').exists()).toBe(false)
    expect(mocks.saveSettings).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it('shows an explicit choice without silently selecting the first available model', async () => {
    mocks.getSettings.mockResolvedValue(settingsFor('http://local.test'))
    mocks.listOllama.mockResolvedValue(['model-one', 'model-two'])
    const wrapper = mountManager()
    await flushMicrotasks()
    const selectors = wrapper.findAll<HTMLSelectElement>('.mm-select').slice(0, 2)
    for (const selector of selectors) {
      expect(selector.element.value).toBe('')
      expect(selector.get('option[value=""]').text()).toBe('modelManager.chooseModel')
    }
    expect(mocks.saveSettings).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it('signals saved setup only after settings are actually persisted', async () => {
    const settings = settingsFor('http://local.test')
    settings.sessionOllamaModel = 'chosen:7b'
    mocks.getSettings.mockResolvedValue(settings)
    mocks.listOllama.mockResolvedValue(['chosen:7b'])
    mocks.saveSettings.mockRejectedValueOnce(new Error('[IO_ERROR] could not save'))
    const wrapper = mountManager()
    await flushMicrotasks()
    await wrapper.get('.mm-footer ui-button-stub').trigger('click')
    await flushMicrotasks()
    expect(wrapper.emitted('saved')).toBeUndefined()
    await wrapper.get('.mm-footer ui-button-stub').trigger('click')
    await flushMicrotasks()
    expect(wrapper.emitted('saved')).toEqual([['role-a']])
    wrapper.unmount()
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
    expect(mocks.showToast).toHaveBeenCalledWith('error', 'global unavailable', '')

    wrapper.unmount()
  })

  it('surfaces a folder scan failure and allows another scan without leaving the form blocked', async () => {
    const settings = settingsFor('http://local.test')
    settings.localModelsDir = 'E:/isolated-model-fixture'
    mocks.getSettings.mockResolvedValue(settings)
    mocks.scan.mockRejectedValueOnce(new Error('[IO_ERROR] scan denied')).mockResolvedValueOnce([])
    const wrapper = mountManager(true)
    await flushMicrotasks()
    const scan = wrapper.findAll('button').find(button => button.text() === 'modelManager.scanFolderModels')!
    await scan.trigger('click')
    await flushMicrotasks()
    expect(mocks.showToast).toHaveBeenCalledWith('error', expect.any(String), expect.stringContaining('scan denied'))
    expect(scan.attributes('disabled')).toBeUndefined()
    await scan.trigger('click')
    await flushMicrotasks()
    expect(mocks.scan).toHaveBeenCalledTimes(2)
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

  it.each([
    ['input[placeholder="modelManager.remoteUrlPlaceholder"]', 'https://unsaved.test/v1'],
    ['input[list="mm-cloud-model-list"]', 'unsaved-model'],
    ['input[type="password"]', 'synthetic-new-key'],
  ])('requires saving changed cloud fields before testing: %s', async (selector, value) => {
    mocks.getSettings.mockResolvedValue(cloudSettings())
    const wrapper = mountManager(true)
    await flushMicrotasks()
    const probe = wrapper.findAll('.mm-footer button')[1]
    expect(probe.attributes('disabled')).toBeUndefined()
    await wrapper.get(selector).setValue(value)
    expect(probe.attributes('disabled')).toBeDefined()
    expect(wrapper.text()).toContain('modelManager.probeNeedSave')
    await probe.trigger('click')
    expect(mocks.probe).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it('also rejects an unsaved test from the handler and re-enables it after reverting the form', async () => {
    mocks.getSettings.mockResolvedValue(cloudSettings())
    const wrapper = mountManager()
    await flushMicrotasks()
    const url = wrapper.get('input[placeholder="modelManager.remoteUrlPlaceholder"]')
    await url.setValue('https://unsaved.test/v1')
    await wrapper.findAll('.mm-footer ui-button-stub')[1].trigger('click')
    expect(mocks.probe).not.toHaveBeenCalled()
    expect(mocks.showToast).toHaveBeenCalledWith('info', 'modelManager.probeNeedSave')
    await url.setValue('https://saved.test/v1')
    await wrapper.findAll('.mm-footer ui-button-stub')[1].trigger('click')
    await flushMicrotasks()
    expect(mocks.probe).toHaveBeenCalledWith('role-a')
    wrapper.unmount()
  })

  it('tests the newly saved cloud configuration and clears its unsaved indication after reload', async () => {
    mocks.getSettings.mockResolvedValue(cloudSettings())
    const wrapper = mountManager(true)
    await flushMicrotasks()
    await wrapper.get('input[placeholder="modelManager.remoteUrlPlaceholder"]').setValue('https://saved-next.test/v1')
    mocks.getSettings.mockResolvedValue({ ...cloudSettings(), remoteUrl: 'https://saved-next.test/v1' })
    await wrapper.findAll('.mm-footer button')[0].trigger('click')
    await flushMicrotasks()
    expect(mocks.saveSettings).toHaveBeenCalledWith(expect.objectContaining({ roleId: 'role-a', remoteUrl: 'https://saved-next.test/v1' }))
    expect(mocks.probe).toHaveBeenCalledWith('role-a')
    expect(wrapper.findAll('.mm-footer button')[1].attributes('disabled')).toBeUndefined()
    expect(wrapper.text()).not.toContain('modelManager.probeNeedSave')
    wrapper.unmount()
  })

  it('does not treat the previous role settings as saved configuration after a new role load fails', async () => {
    mocks.getSettings.mockResolvedValueOnce(cloudSettings()).mockRejectedValueOnce(new Error('settings unavailable'))
    const wrapper = mountManager(true)
    await flushMicrotasks()
    const { useRoleStore } = await import('@oclive/shared/stores/roleStore')
    useRoleStore().currentRoleId = 'role-b'
    await flushMicrotasks()
    expect(wrapper.findAll('.mm-footer button')[1].attributes('disabled')).toBeDefined()
    expect(mocks.probe).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it.each([
    ['manual', 'success'],
    ['manual', 'failure'],
    ['saved', 'success'],
    ['saved', 'failure'],
  ])('ignores %s probe %s after unmount without refreshing the disposed form', async (source, outcome) => {
    mocks.getSettings.mockResolvedValue(cloudSettings())
    const result = deferred<string>()
    mocks.probe.mockReturnValueOnce(result.promise)
    const wrapper = mountManager(true)
    await flushMicrotasks()
    await wrapper.findAll('.mm-footer button')[source === 'saved' ? 0 : 1].trigger('click')
    await flushMicrotasks()
    expect(mocks.probe).toHaveBeenCalledWith('role-a')
    wrapper.unmount()
    mocks.showToast.mockClear()
    if (outcome === 'success')
      result.resolve('ok')
    else
      result.reject(new Error('synthetic probe failure'))
    await flushMicrotasks()
    expect(mocks.showToast).not.toHaveBeenCalled()
    expect(wrapper.emitted('saved')).toBeUndefined()
    expect(mocks.getSettings).toHaveBeenCalledTimes(1)
  })

  it('keeps the new role probe busy when the previous role probe finishes last', async () => {
    mocks.getSettings.mockResolvedValue(cloudSettings())
    const previous = deferred<string>()
    const current = deferred<string>()
    mocks.probe.mockReturnValueOnce(previous.promise).mockReturnValueOnce(current.promise)
    const wrapper = mountManager(true)
    await flushMicrotasks()
    await wrapper.findAll('.mm-footer button')[1].trigger('click')
    const { useRoleStore } = await import('@oclive/shared/stores/roleStore')
    useRoleStore().currentRoleId = 'role-b'
    await flushMicrotasks()
    expect(wrapper.findAll('.mm-footer button')[1].attributes('disabled')).toBeUndefined()
    await wrapper.findAll('.mm-footer button')[1].trigger('click')
    expect(mocks.probe).toHaveBeenLastCalledWith('role-b')
    previous.resolve('old ok')
    await flushMicrotasks()
    expect(mocks.showToast).not.toHaveBeenCalledWith('success', 'modelManager.probeOk')
    expect(wrapper.findAll('.mm-footer button')[1].text()).toBe('modelManager.probing')
    current.resolve('current ok')
    await flushMicrotasks()
    expect(mocks.showToast).toHaveBeenCalledTimes(1)
    expect(mocks.showToast).toHaveBeenCalledWith('success', 'modelManager.probeOk')
  })

  it('does not accept an earlier probe after switching away and returning to the same role', async () => {
    mocks.getSettings.mockResolvedValue(cloudSettings())
    const result = deferred<string>()
    mocks.probe.mockReturnValueOnce(result.promise)
    const wrapper = mountManager(true)
    await flushMicrotasks()
    await wrapper.findAll('.mm-footer button')[1].trigger('click')
    const { useRoleStore } = await import('@oclive/shared/stores/roleStore')
    useRoleStore().currentRoleId = 'role-b'
    await flushMicrotasks()
    useRoleStore().currentRoleId = 'role-a'
    await flushMicrotasks()
    result.resolve('old ok')
    await flushMicrotasks()
    expect(mocks.showToast).not.toHaveBeenCalled()
    expect(wrapper.findAll('.mm-footer button')[1].attributes('disabled')).toBeUndefined()
  })

  it('preserves new edits made during the automatic post-save probe', async () => {
    mocks.getSettings.mockResolvedValue(cloudSettings())
    const result = deferred<string>()
    mocks.probe.mockReturnValueOnce(result.promise)
    const wrapper = mountManager(true)
    await flushMicrotasks()
    await wrapper.findAll('.mm-footer button')[0].trigger('click')
    await flushMicrotasks()
    await wrapper.get('input[list="mm-cloud-model-list"]').setValue('new-unsaved-model')
    mocks.showToast.mockClear()
    result.resolve('old ok')
    await flushMicrotasks()
    expect(mocks.showToast).not.toHaveBeenCalled()
    expect(wrapper.emitted('saved')).toBeUndefined()
    expect(mocks.getSettings).toHaveBeenCalledTimes(1)
    expect(wrapper.get<HTMLInputElement>('input[list="mm-cloud-model-list"]').element.value).toBe('new-unsaved-model')
    expect(wrapper.findAll('.mm-footer button')[0].attributes('disabled')).toBeUndefined()
  })

  it('preserves edits made while saving and does not probe them as if they were saved', async () => {
    mocks.getSettings.mockResolvedValue(cloudSettings())
    const result = deferred<Record<string, unknown>>()
    mocks.saveSettings.mockReturnValueOnce(result.promise)
    const wrapper = mountManager(true)
    await flushMicrotasks()
    await wrapper.findAll('.mm-footer button')[0].trigger('click')
    await wrapper.get('input[list="mm-cloud-model-list"]').setValue('new-unsaved-model')
    result.resolve({})
    await flushMicrotasks()
    expect(mocks.probe).not.toHaveBeenCalled()
    expect(wrapper.emitted('saved')).toBeUndefined()
    expect(wrapper.get<HTMLInputElement>('input[list="mm-cloud-model-list"]').element.value).toBe('new-unsaved-model')
    expect(wrapper.findAll('.mm-footer button')[1].attributes('disabled')).toBeDefined()
  })

  it('still reports a current post-save probe failure as a warning and finishes the successful save', async () => {
    mocks.getSettings.mockResolvedValue(cloudSettings())
    mocks.probe.mockRejectedValueOnce(new Error('synthetic probe failure'))
    const wrapper = mountManager(true)
    await flushMicrotasks()
    await wrapper.findAll('.mm-footer button')[0].trigger('click')
    await flushMicrotasks()
    expect(mocks.showToast).toHaveBeenCalledWith('warning', expect.any(String), expect.any(String))
    expect(wrapper.emitted('saved')).toEqual([['role-a']])
    expect(wrapper.findAll('.mm-footer button')[0].attributes('disabled')).toBeUndefined()
  })
})

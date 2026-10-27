// @vitest-environment jsdom
import type { ComposerTranslation } from 'vue-i18n'
import { useChatStore } from '@oclive/shared/stores/chatStore'
import { useKernelConnectionStore } from '@oclive/shared/stores/kernelConnectionStore'
import { useRoleStore } from '@oclive/shared/stores/roleStore'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent, h, ref } from 'vue'
import { useAppBootstrap } from '../composables/useAppBootstrap'

const mocks = vi.hoisted(() => ({ list: vi.fn(), load: vi.fn(), notify: vi.fn(), reporter: vi.fn() }))
vi.mock('@oclive/shared/api', async original => ({
  ...await original<typeof import('@oclive/shared/api')>(),
  listRoles: mocks.list,
  loadRole: mocks.load,
  setErrorReporter: mocks.reporter,
}))
vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn(async () => () => {}) }))

function start() {
  const pinia = createPinia()
  vi.spyOn(useChatStore(pinia), 'hydrateFromStorage').mockResolvedValue(undefined)
  const role = useRoleStore(pinia)
  role.currentRoleId = 'previous-role'
  let bootstrap!: ReturnType<typeof useAppBootstrap>
  const wrapper = mount(defineComponent({
    setup() {
      bootstrap = useAppBootstrap({
        deferRoleActivation: true,
        advancedFeaturesEnabled: () => false,
        showToast: mocks.notify,
        t: ((key: string) => key) as ComposerTranslation,
        openPluginManagerPanel: vi.fn(),
        localePreference: ref('system'),
        syncBrowserChromeFromLocale: vi.fn(),
        scheduleRefreshSplitLayout: vi.fn(),
        refreshSplitLayout: vi.fn(),
      })
      return () => h('div', bootstrap.startupState.value)
    },
  }), { global: { plugins: [pinia] } })
  return {
    role,
    kernelConnection: useKernelConnectionStore(pinia),
    wrapper,
    get bootstrap() {
      return bootstrap
    },
  }
}

describe('role-library-first startup', () => {
  beforeEach(() => vi.resetAllMocks())
  it('lists existing roles without activating the saved role', async () => {
    mocks.list.mockResolvedValue([{ id: 'mumu', name: 'Mumu' }])
    const app = start()
    await flushPromises()
    expect(app.role.roles).toHaveLength(1)
    expect(app.role.currentRoleId).toBe('')
    expect(app.bootstrap.startupState.value).toBe('ready')
    expect(mocks.load).not.toHaveBeenCalled()
    app.wrapper.unmount()
  })
  it('treats an empty role library as ready, not an error or endless loading', async () => {
    mocks.list.mockResolvedValue([])
    const app = start()
    await flushPromises()
    expect(app.bootstrap.startupState.value).toBe('ready')
    expect(mocks.notify).not.toHaveBeenCalled()
    app.wrapper.unmount()
  })
  it('exposes a retryable loading failure', async () => {
    mocks.list.mockRejectedValueOnce(new Error('library unavailable')).mockResolvedValue([])
    const app = start()
    await flushPromises()
    expect(app.bootstrap.startupState.value).toBe('error')
    expect(app.bootstrap.startupError.value).toBe('library unavailable')
    await app.bootstrap.initialize()
    expect(app.bootstrap.startupState.value).toBe('ready')
    expect(app.bootstrap.startupError.value).toBe('')
    app.wrapper.unmount()
  })

  it('keeps the connection indicator responsible for cold-start probe failures', async () => {
    mocks.list.mockResolvedValue([])
    const app = start()
    await flushPromises()
    const report = mocks.reporter.mock.calls[0][0]
    report({ code: 'KERNEL_OFFLINE', raw: '[KERNEL_OFFLINE] warming up', message: 'Kernel offline' })
    expect(mocks.notify).not.toHaveBeenCalled()
    app.wrapper.unmount()
  })

  it('retains raw diagnostics in global errors once startup has finished', async () => {
    mocks.list.mockResolvedValue([])
    const app = start()
    await flushPromises()
    app.kernelConnection.phase = 'ready'
    const report = mocks.reporter.mock.calls[0][0]
    report({ code: 'KERNEL_OFFLINE', raw: '[KERNEL_OFFLINE] listener not running', message: 'Kernel offline' })
    expect(mocks.notify).toHaveBeenLastCalledWith('error', expect.any(String), expect.stringContaining('listener not running'))
    report({ code: 'IO_ERROR', raw: '[IO_ERROR] directory inaccessible', message: 'File operation failed' })
    expect(mocks.notify).toHaveBeenLastCalledWith('error', expect.any(String), expect.stringContaining('directory inaccessible'))
    app.wrapper.unmount()
  })
})

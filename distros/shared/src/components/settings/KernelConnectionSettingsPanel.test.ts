// @vitest-environment jsdom
import type { KernelConnectionStatus } from '@oclive/shared/api/kernel'
import { i18n } from '@oclive/shared/i18n'
import { useKernelConnectionStore } from '@oclive/shared/stores/kernelConnectionStore'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia } from 'pinia'
import { expect, it, vi } from 'vitest'
import { nextTick } from 'vue'
import KernelConnectionSettingsPanel from './KernelConnectionSettingsPanel.vue'

const api = vi.hoisted(() => ({
  getKernelConnectionStatus: vi.fn(),
  getKernelDiagnostics: vi.fn(),
  reconnectKernel: vi.fn(),
}))
vi.mock('@oclive/shared/api/kernel', () => api)
vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn(async () => () => {}) }))

it('shows the cold-start failure and collapses diagnostic details until needed', async () => {
  i18n.global.locale.value = 'en-US'
  const offline: KernelConnectionStatus = {
    mode: 'offline',
    baseUrl: 'http://127.0.0.1:8420',
    port: 8420,
    binaryPath: null,
    kernelTier: null,
    healthy: false,
    statusMessage: 'The database fixture/data/app.db is missing and chat mirrors remain. Authorization: Bearer fixture-secret',
  }
  const diagnostics = {
    status: offline,
    sharedRuntimePath: '',
    sharedRuntimeExists: false,
    sharedRuntimeModifiedMs: null,
    healthJson: null,
  }
  api.getKernelConnectionStatus.mockResolvedValue(offline)
  api.getKernelDiagnostics.mockResolvedValue(diagnostics)
  const pinia = createPinia()
  const wrapper = mount(KernelConnectionSettingsPanel, { props: { active: true }, global: { plugins: [pinia, i18n] } })
  await flushPromises()

  const notice = wrapper.get('.error-notice')
  expect(notice.get('.error-notice-heading').text()).toContain('The local kernel could not start.')
  expect(notice.get('.error-notice-heading').text()).not.toContain('app.db')
  expect(notice.get('details').attributes('open')).toBeUndefined()
  expect(notice.get('pre').text()).toContain('fixture/data/app.db is missing')
  expect(notice.get('pre').text()).not.toContain('fixture-secret')

  api.reconnectKernel.mockRejectedValueOnce(new Error('[STARTUP_HEALTH_FAILED] history database missing'))
  await wrapper.get('.kernel-panel__actions button:last-child').trigger('click')
  await flushPromises()
  expect(wrapper.findAll('.error-notice')).toHaveLength(1)

  let finishRefresh!: (value: typeof diagnostics) => void
  api.getKernelDiagnostics.mockImplementationOnce(() => new Promise((resolve) => {
    finishRefresh = resolve
  }))
  await wrapper.get('.kernel-panel__actions button:first-child').trigger('click')
  const store = useKernelConnectionStore(pinia)
  store.applyStatus({ ...offline, mode: 'attached', healthy: true, statusMessage: undefined })
  await nextTick()
  finishRefresh(diagnostics)
  await flushPromises()
  expect(wrapper.find('.error-notice').exists()).toBe(false)
  expect(wrapper.get('.kernel-panel__facts').text()).toContain('attached')
  expect(wrapper.get('.kernel-panel__facts').text()).not.toContain('Unreachable')
  wrapper.unmount()
})

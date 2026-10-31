// @vitest-environment jsdom
import { i18n } from '@oclive/shared/i18n'
import { flushPromises, mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import SupportDiagnostics from './SupportDiagnostics.vue'

const api = vi.hoisted(() => ({ logs: vi.fn(), check: vi.fn(), write: vi.fn(), save: vi.fn(), copy: vi.fn() }))
vi.mock('@oclive/shared/api/diagnostics', () => ({ readSupportLogs: api.logs, runEnvironmentDiagnostics: api.check }))
vi.mock('@oclive/shared/api/desktop', () => ({ writeUserTextFile: api.write }))
vi.mock('@tauri-apps/plugin-dialog', () => ({ save: api.save }))
vi.mock('@oclive/shared/stores/kernelConnectionStore', () => ({ useKernelConnectionStore: () => ({ status: { healthy: true, mode: 'spawned' } }) }))

const basicCredential = btoa('review-user:synthetic-password')
const snapshot = { directory: 'isolated/logs', files: [{ name: 'oclive.log', sizeBytes: 600, modifiedAtEpochMs: 1 }], selectedFile: 'oclive.log', truncated: true, text: `INFO ready\nWARN slow\nERROR request api_key=secret-value; Authorization: Basic ${basicCredential}` }
function start() {
  return mount(SupportDiagnostics, { props: { active: true }, global: { plugins: [i18n], stubs: { KernelConnectionSettingsPanel: true } } })
}

describe('help diagnostics interactions', () => {
  beforeEach(() => {
    vi.resetAllMocks()
    i18n.global.locale.value = 'en-US'
    api.logs.mockResolvedValue(snapshot)
    Object.defineProperty(navigator, 'clipboard', { configurable: true, value: { writeText: api.copy } })
  })

  it('shows bounded log tails with level/search filters and redacts display/export', async () => {
    const wrapper = start()
    await flushPromises()
    await wrapper.find('#support-tab-logs').trigger('click')
    expect(wrapper.findAll('.support-log-line')).toHaveLength(3)
    expect(wrapper.text()).not.toContain('secret-value')
    expect(wrapper.text()).not.toContain(basicCredential)
    expect(wrapper.find('.support-limit').text()).toContain('omitted')
    await wrapper.findAll('select')[1].setValue('error')
    expect(wrapper.findAll('.support-log-line')).toHaveLength(1)
    await wrapper.find('input[type=search]').setValue('missing')
    expect(wrapper.findAll('.support-log-line')).toHaveLength(0)
    const copy = wrapper.findAll('button').find(button => button.text() === 'Copy diagnostic summary')!
    await copy.trigger('click')
    await flushPromises()
    expect(api.copy).toHaveBeenCalledWith(expect.stringContaining('ERROR request'))
    expect(api.copy.mock.calls[0][0]).not.toContain('secret-value')
    expect(api.copy.mock.calls[0][0]).not.toContain(basicCredential)
    api.save.mockResolvedValueOnce('isolated/report.txt')
    const exportButton = wrapper.findAll('button').find(button => button.text() === 'Save diagnostic report')!
    await exportButton.trigger('click')
    await flushPromises()
    expect(api.write).toHaveBeenCalledWith('isolated/report.txt', expect.stringContaining('[redacted]'))
    expect(api.write.mock.calls[0][1]).not.toContain(basicCredential)
    wrapper.unmount()
  })

  it('exposes read failures and recovers with a manual refresh', async () => {
    api.logs.mockRejectedValueOnce(new Error('[IO_ERROR] log locked'))
    const wrapper = start()
    await flushPromises()
    expect(wrapper.find('[role=alert]').exists()).toBe(true)
    await wrapper.find('#support-tab-logs').trigger('click')
    const refresh = wrapper.findAll('button').find(button => button.text() === 'Refresh logs')!
    await refresh.trigger('click')
    await flushPromises()
    expect(wrapper.find('[role=alert]').exists()).toBe(false)
    expect(wrapper.findAll('.support-log-line')).toHaveLength(3)
    wrapper.unmount()
  })

  it('ignores late responses after closing and does not write a cancelled export', async () => {
    let finish!: (value: typeof snapshot) => void
    api.logs.mockImplementationOnce(() => new Promise((resolve) => {
      finish = resolve
    }))
    const wrapper = start()
    await wrapper.setProps({ active: false })
    finish(snapshot)
    await flushPromises()
    await wrapper.find('#support-tab-logs').trigger('click')
    expect(wrapper.findAll('.support-log-line')).toHaveLength(0)
    api.save.mockResolvedValueOnce(null)
    await wrapper.findAll('button').find(button => button.text() === 'Save diagnostic report')!.trigger('click')
    await flushPromises()
    expect(api.write).not.toHaveBeenCalled()
    wrapper.unmount()
  })
})

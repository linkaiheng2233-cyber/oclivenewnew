// @vitest-environment jsdom
import { flushPromises, mount } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import Win98TitleBar from './Win98TitleBar.vue'

const mocks = vi.hoisted(() => ({ minimize: vi.fn(), toggleMaximize: vi.fn(), close: vi.fn() }))
vi.mock('@oclive/shared/composables/useEasterEggSkin', () => ({ useEasterEggSkin: () => ({ win98Enabled: true }) }))
vi.mock('@oclive/shared/stores/roleStore', () => ({ useRoleStore: () => ({ roles: [{ id: 'mumu', name: 'Mumu' }], currentRoleId: 'mumu' }) }))
vi.mock('@tauri-apps/api/webviewWindow', () => ({ getCurrentWebviewWindow: () => mocks }))

describe('win98 caption window actions', () => {
  beforeEach(() => {
    vi.resetAllMocks()
    Object.defineProperty(window, '__TAURI_INTERNALS__', { configurable: true, value: {} })
  })
  afterEach(() => Reflect.deleteProperty(window, '__TAURI_INTERNALS__'))

  it('routes each caption button to its own native window action', async () => {
    const wrapper = mount(Win98TitleBar)
    expect(wrapper.text()).toContain('A.I.Live Chat Pro - Mumu')
    for (const label of ['Minimize', 'Maximize', 'Close']) {
      await wrapper.get(`button[aria-label="${label}"]`).trigger('click')
      await flushPromises()
    }
    expect(mocks.minimize).toHaveBeenCalledTimes(1)
    expect(mocks.toggleMaximize).toHaveBeenCalledTimes(1)
    expect(mocks.close).toHaveBeenCalledTimes(1)
    wrapper.unmount()
  })

  it('maximizes by double-clicking the caption but not by double-clicking a control', async () => {
    const wrapper = mount(Win98TitleBar)
    await wrapper.get('.win98-titlebar__title').trigger('dblclick')
    await flushPromises()
    expect(mocks.toggleMaximize).toHaveBeenCalledTimes(1)
    await wrapper.get('button[aria-label="Close"]').trigger('dblclick')
    await flushPromises()
    expect(mocks.toggleMaximize).toHaveBeenCalledTimes(1)
    wrapper.unmount()
  })

  it('ignores native controls in a browser without throwing', async () => {
    Reflect.deleteProperty(window, '__TAURI_INTERNALS__')
    const wrapper = mount(Win98TitleBar)
    await wrapper.get('button[aria-label="Close"]').trigger('click')
    await flushPromises()
    expect(mocks.close).not.toHaveBeenCalled()
    wrapper.unmount()
  })
})

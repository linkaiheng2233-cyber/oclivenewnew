// @vitest-environment jsdom
import { flushPromises, mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import HotkeyHost from './HotkeyHost.vue'

const mocks = vi.hoisted(() => ({ listen: vi.fn(), stop: vi.fn() }))
vi.mock('@tauri-apps/api/event', () => ({ listen: mocks.listen }))
vi.mock('vue-i18n', () => ({ useI18n: () => ({ t: (key: string) => key }) }))
vi.mock('@oclive/shared/stores/pluginStore', () => ({ usePluginStore: () => ({ bootstrapUiSlots: [] }) }))
vi.mock('@oclive/shared/composables/usePluginFrameBridge', () => ({
  usePluginFrameBridge: () => ({ bindPluginFrame: vi.fn(), framePermissions: vi.fn(), onPluginFrameLoad: vi.fn() }),
}))

describe('optional hotkey host disposal', () => {
  beforeEach(() => vi.clearAllMocks())

  it('unregisters a listener that finishes registering after the feature is closed', async () => {
    let resolveListener: ((stop: () => void) => void) | undefined
    mocks.listen.mockReturnValue(new Promise<() => void>((resolve) => {
      resolveListener = resolve
    }))
    const wrapper = mount(HotkeyHost)
    wrapper.unmount()
    resolveListener!(mocks.stop)
    await flushPromises()
    expect(mocks.stop).toHaveBeenCalledTimes(1)
  })

  it('unregisters the active listener once on ordinary close', async () => {
    mocks.listen.mockResolvedValue(mocks.stop)
    const wrapper = mount(HotkeyHost)
    await flushPromises()
    wrapper.unmount()
    expect(mocks.stop).toHaveBeenCalledTimes(1)
  })
})

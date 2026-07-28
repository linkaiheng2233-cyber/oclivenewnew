// @vitest-environment jsdom

import { mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent } from 'vue'
import { invalidateVoiceRuntimeConfig, useVoiceAutoTts } from './useVoiceAutoTts'

const mocks = vi.hoisted(() => ({
  handlers: new Map<string, (payload: unknown) => unknown>(),
  directoryInvoke: vi.fn(),
  markSettled: vi.fn(),
  showToast: vi.fn(),
  roleStore: { currentRoleId: 'new-role' },
  pluginDisabled: true,
  getSettings: vi.fn(),
  invokeFriendly: vi.fn(),
}))

vi.mock('@oclive/shared/api', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@oclive/shared/api')>()
  return {
    ...actual,
    directoryPluginInvoke: mocks.directoryInvoke,
    getPluginSettingsUi: mocks.getSettings,
  }
})

vi.mock('@oclive/shared/api/helpers', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@oclive/shared/api/helpers')>()
  return {
    ...actual,
    invokeWithFriendlyError: mocks.invokeFriendly,
  }
})

vi.mock('@oclive/shared/composables/useVoiceExpansionWarm', () => ({
  resetVoiceExpansionWarmSchedule: vi.fn(),
  resolveVoiceSidecarEndpoint: vi.fn(),
  scheduleVoiceExpansionWarm: vi.fn(async () => undefined),
}))

vi.mock('@oclive/shared/lib/hostEventBus', () => ({
  hostEventBus: {
    off: vi.fn((event: string) => mocks.handlers.delete(event)),
    on: vi.fn((event: string, handler: (payload: unknown) => unknown) => {
      mocks.handlers.set(event, handler)
    }),
  },
}))

vi.mock('@oclive/shared/lib/voicePlaybackSettlement', () => ({
  markVoicePlaybackSettled: mocks.markSettled,
}))

vi.mock('@oclive/shared/stores/pluginStore', () => ({
  usePluginStore: () => ({
    isPluginDisabled: () => mocks.pluginDisabled,
  }),
}))

vi.mock('@oclive/shared/stores/roleStore', () => ({
  useRoleStore: () => mocks.roleStore,
}))

const Harness = defineComponent({
  setup() {
    useVoiceAutoTts({ showToast: mocks.showToast })
    return () => null
  },
})

describe('voice auto TTS ownership', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    invalidateVoiceRuntimeConfig()
    mocks.handlers.clear()
    mocks.roleStore.currentRoleId = 'new-role'
    mocks.pluginDisabled = true
    mocks.getSettings.mockResolvedValue({ config: {} })
    mocks.invokeFriendly.mockResolvedValue('')
  })

  it('settles but never speaks a late message from the previous role', async () => {
    const wrapper = mount(Harness)
    const onMessageSent = mocks.handlers.get('message:sent')
    expect(onMessageSent).toBeTypeOf('function')

    await onMessageSent?.({
      reply: 'old role dialogue',
      reply_aside: 'silent narration',
      role_id: 'old-role',
      turn_id: 'old-turn',
    })

    expect(mocks.directoryInvoke).not.toHaveBeenCalled()
    expect(mocks.markSettled)
      .toHaveBeenCalledWith('old-turn', 'disabled')
    expect(mocks.showToast).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it('does not warm or speak a role omitted from the explicit role map', async () => {
    mocks.pluginDisabled = false
    mocks.roleStore.currentRoleId = 'gentle-landlady'
    mocks.getSettings.mockResolvedValue({
      config: {
        tts_expansion_enabled: true,
        auto_tts: true,
        role_tts_enabled: { mumu: true },
        tts_profile: 'bundled-cosyvoice2-zh',
      },
    })
    mocks.directoryInvoke.mockImplementation(async (
      _pluginId: string,
      method: string,
    ) => method === 'voice.list_profiles' ? { profiles: [] } : { ok: true })

    const wrapper = mount(Harness)
    await vi.waitFor(() => expect(mocks.getSettings).toHaveBeenCalled())
    const onSubmit = mocks.handlers.get('message:submit')
    onSubmit?.({ role_id: 'gentle-landlady', stream_id: 'stream-1' })
    await new Promise(resolve => setTimeout(resolve, 0))

    const methods = mocks.directoryInvoke.mock.calls.map(([, method]) => method)
    expect(methods).not.toContain('voice.read_role_profile')
    expect(methods).not.toContain('voice.warm')
    expect(methods).not.toContain('voice.speak')
    expect(mocks.showToast).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it('does not use the global voice for an enabled role without voice_profile.json', async () => {
    mocks.pluginDisabled = false
    mocks.roleStore.currentRoleId = 'gentle-landlady'
    mocks.getSettings.mockResolvedValue({
      config: {
        tts_expansion_enabled: true,
        auto_tts: true,
        role_tts_enabled: { 'gentle-landlady': true },
        tts_profile: 'bundled-cosyvoice2-zh',
      },
    })
    mocks.invokeFriendly.mockResolvedValue('D:/roles/gentle-landlady')
    mocks.directoryInvoke.mockImplementation(async (
      _pluginId: string,
      method: string,
    ) => {
      if (method === 'voice.list_profiles')
        return { profiles: [] }
      if (method === 'voice.read_role_profile')
        return { ok: true, profile: null }
      return { ok: true }
    })

    const wrapper = mount(Harness)
    await vi.waitFor(() => {
      expect(mocks.directoryInvoke).toHaveBeenCalledWith(
        expect.anything(),
        'voice.read_role_profile',
        expect.anything(),
      )
    })
    const onSubmit = mocks.handlers.get('message:submit')
    onSubmit?.({ role_id: 'gentle-landlady', stream_id: 'stream-2' })
    await new Promise(resolve => setTimeout(resolve, 0))

    const methods = mocks.directoryInvoke.mock.calls.map(([, method]) => method)
    expect(methods).not.toContain('voice.warm')
    expect(methods).not.toContain('voice.speak')
    expect(mocks.showToast).not.toHaveBeenCalled()
    wrapper.unmount()
  })
})

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
  resolveSidecarEndpoint: vi.fn(),
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
  resolveVoiceSidecarEndpoint: (...args: unknown[]) => mocks.resolveSidecarEndpoint(...args),
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

vi.mock('@oclive/shared/utils/cosyvoiceStreamPlayback', async (importOriginal) => {
  const actual = await importOriginal<
    typeof import('@oclive/shared/utils/cosyvoiceStreamPlayback')
  >()
  return {
    ...actual,
    ensureVoiceAudioReady: vi.fn(async () => undefined),
  }
})

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
    mocks.resolveSidecarEndpoint.mockResolvedValue(null)
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

  it('defers GPU-denied streamed speech and retries by RPC after final text', async () => {
    mocks.pluginDisabled = false
    mocks.roleStore.currentRoleId = 'mumu'
    mocks.getSettings.mockResolvedValue({
      config: {
        tts_expansion_enabled: true,
        auto_tts: true,
        role_tts_enabled: { mumu: true },
        tts_profile: 'bundled-cosyvoice2-zh',
        synth_provider: 'bundled',
      },
    })
    mocks.invokeFriendly.mockResolvedValue('D:/roles/mumu')
    let speakAttempts = 0
    mocks.directoryInvoke.mockImplementation(async (
      _pluginId: string,
      method: string,
    ) => {
      if (method === 'voice.list_profiles')
        return { profiles: [] }
      if (method === 'voice.read_role_profile') {
        return {
          ok: true,
          profile: {
            synth_profile: 'bundled-cosyvoice2-zh',
          },
        }
      }
      if (method === 'voice.build_directive') {
        return {
          ok: true,
          directive: {
            synth_profile: 'bundled-cosyvoice2-zh',
          },
        }
      }
      if (method === 'voice.speak') {
        speakAttempts += 1
        return speakAttempts === 1
          ? { ok: false, reason: 'gpu_admission_denied' }
          : { ok: false, reason: 'test_retry_observed' }
      }
      return { ok: true }
    })

    const wrapper = mount(Harness)
    await vi.waitFor(() => expect(mocks.getSettings).toHaveBeenCalled())
    mocks.handlers.get('message:submit')?.({
      role_id: 'mumu',
      stream_id: 'stream-gpu',
    })
    mocks.handlers.get('com.oclive.voice:stream-sentence')?.({
      sentence: '第一段',
      role_id: 'mumu',
      stream_id: 'stream-gpu',
    })
    await vi.waitFor(() => expect(speakAttempts).toBe(1))

    await mocks.handlers.get('message:sent')?.({
      reply: '第一段，第二段。',
      role_id: 'mumu',
      stream_id: 'stream-gpu',
      turn_id: 'turn-gpu',
    })

    expect(speakAttempts).toBe(2)
    expect(mocks.showToast).toHaveBeenCalledWith(
      'info',
      expect.stringContaining('等待本轮文本生成完成'),
    )
    expect(mocks.markSettled).toHaveBeenCalledWith('turn-gpu', 'error')
    wrapper.unmount()
  })
})

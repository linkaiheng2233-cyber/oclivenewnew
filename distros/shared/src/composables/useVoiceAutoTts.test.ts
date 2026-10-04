// @vitest-environment jsdom

import { mount } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
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

/**
 * In-memory audio double: no real audio device, no decoding. `play()` reports a completed
 * playback on a microtask so the consumer's normal success path can be asserted end to end.
 */
class InMemoryAudio extends EventTarget {
  src = ''
  paused = true

  play(): Promise<void> {
    this.paused = false
    queueMicrotask(() => this.dispatchEvent(new Event('ended')))
    return Promise.resolve()
  }

  pause(): void {
    this.paused = true
  }

  load(): void {}

  removeAttribute(name: string): void {
    if (name === 'src')
      this.src = ''
  }
}

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

  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('does not load or prewarm voice resources for an explicitly text-only submitted turn', async () => {
    const wrapper = mount(Harness)
    await Promise.resolve()
    invalidateVoiceRuntimeConfig()
    mocks.pluginDisabled = false
    mocks.getSettings.mockResolvedValue({
      config: { tts_expansion_enabled: true, auto_tts: true, role_tts_enabled: { 'new-role': true } },
    })
    vi.clearAllMocks()
    try {
      mocks.handlers.get('message:submit')?.({ role_id: 'new-role', skip_auto_tts: true })
      await Promise.resolve()
      expect(mocks.getSettings).not.toHaveBeenCalled()
      expect(mocks.invokeFriendly).not.toHaveBeenCalled()
      expect(mocks.directoryInvoke).not.toHaveBeenCalled()
      expect(mocks.resolveSidecarEndpoint).not.toHaveBeenCalled()
      expect(mocks.markSettled).not.toHaveBeenCalled()
    }
    finally {
      wrapper.unmount()
    }
  })

  it('speaks the final authoritative reply once and stays silent until then', async () => {
    vi.stubGlobal('Audio', InMemoryAudio)
    // Raw streamed tokens the bubble may have previewed while the turn was still open.
    const rawTokenPrefix = '清晨的风轻轻吹过树梢。'
    // The host-processed authoritative text deliberately differs from that prefix.
    const authoritativeReply = '风停了。我们回去吧。'
    expect(authoritativeReply).not.toBe(rawTokenPrefix)
    expect(authoritativeReply).not.toContain(rawTokenPrefix)
    expect(rawTokenPrefix).not.toContain(authoritativeReply)

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
    const speakPayloads: { text?: string }[] = []
    mocks.directoryInvoke.mockImplementation(async (
      _pluginId: string,
      method: string,
      payload?: { text?: string },
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
        speakPayloads.push({ text: payload?.text })
        return { ok: true, audio_base64: 'QUJD', audio_mime: 'audio/wav' }
      }
      return { ok: true }
    })

    const wrapper = mount(Harness)
    await vi.waitFor(() => expect(mocks.getSettings).toHaveBeenCalled())

    // The turn is submitted but the authoritative result has not arrived yet.
    mocks.handlers.get('message:submit')?.({ role_id: 'mumu', stream_id: 'stream-ordinary' })
    await new Promise(resolve => setTimeout(resolve, 0))
    expect(speakPayloads).toHaveLength(0)
    expect(mocks.markSettled).not.toHaveBeenCalled()

    const onMessageSent = mocks.handlers.get('message:sent')
    expect(onMessageSent).toBeTypeOf('function')
    // No `stream_id`: this is the post-fix ordinary-reply payload, so the consumer must
    // speak the whole authoritative reply instead of topping up a streamed prefix.
    await onMessageSent?.({
      reply: authoritativeReply,
      role_id: 'mumu',
      turn_id: 'turn-ordinary',
    })

    expect(speakPayloads).toHaveLength(1)
    expect(speakPayloads[0].text).toBe(authoritativeReply)
    expect(speakPayloads[0].text).not.toBe(rawTokenPrefix)
    expect(mocks.markSettled).toHaveBeenCalledTimes(1)
    expect(mocks.markSettled).toHaveBeenCalledWith('turn-ordinary', 'complete')
    expect(mocks.showToast).not.toHaveBeenCalled()
    wrapper.unmount()
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

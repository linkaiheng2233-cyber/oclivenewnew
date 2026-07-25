import { beforeEach, describe, expect, it, vi } from 'vitest'

const invokeMock = vi.fn()

vi.mock('@oclive/shared/api', () => ({
  directoryPluginInvoke: (...args: unknown[]) => invokeMock(...args),
  getPluginSettingsUi: vi.fn(async () => ({
    config: { tts_expansion_enabled: true, tts_profile: 'bundled-cosyvoice2-zh' },
  })),
}))

describe('useVoiceExpansionWarm', () => {
  beforeEach(async () => {
    vi.resetModules()
    invokeMock.mockReset()
  })

  it('resolveVoiceSidecarEndpoint does not await long warm', async () => {
    let warmResolve: ((value: unknown) => void) | null = null
    invokeMock.mockImplementation(async (_id: string, method: string) => {
      if (method === 'voice.warm') {
        return new Promise((resolve) => {
          warmResolve = resolve
        })
      }
      if (method === 'voice.probe_tts') {
        return { ok: true, sidecar_endpoint: 'http://127.0.0.1:50001', warmed: false }
      }
      return { ok: false }
    })

    const mod = await import('./useVoiceExpansionWarm')
    mod.resetVoiceExpansionWarmSchedule()

    const endpointPromise = mod.resolveVoiceSidecarEndpoint(
      'bundled-cosyvoice2-zh',
      'http://127.0.0.1:50000',
      () => false,
    )

    await expect(endpointPromise).resolves.toBe('http://127.0.0.1:50001')
    expect(warmResolve).not.toBeNull()
    warmResolve?.({ ok: true, sidecar_endpoint: 'http://127.0.0.1:50099' })
  })

  it('scheduleVoiceExpansionWarm skips warm when probe reports warmed', async () => {
    invokeMock.mockImplementation(async (_id: string, method: string) => {
      if (method === 'voice.probe_tts') {
        return { ok: true, warmed: true, sidecar_endpoint: 'http://127.0.0.1:50000' }
      }
      if (method === 'voice.warm') {
        throw new Error('warm should not be called')
      }
      return { ok: false }
    })

    const mod = await import('./useVoiceExpansionWarm')
    mod.resetVoiceExpansionWarmSchedule()
    await mod.scheduleVoiceExpansionWarm(() => false)
    expect(invokeMock).not.toHaveBeenCalledWith(
      expect.anything(),
      'voice.warm',
      expect.anything(),
    )
  })

  it('prepares a role directive even when the model is already warmed', async () => {
    invokeMock.mockImplementation(async (_id: string, method: string) => {
      if (method === 'voice.probe_tts') {
        return { ok: true, warmed: true, sidecar_endpoint: 'http://127.0.0.1:50000' }
      }
      if (method === 'voice.warm') {
        return { ok: true, warmed: true, prompt_prepared: true }
      }
      return { ok: false }
    })

    const mod = await import('./useVoiceExpansionWarm')
    mod.resetVoiceExpansionWarmSchedule()
    await mod.scheduleVoiceExpansionWarm(() => false, {
      profile: 'bundled-cosyvoice2-zh',
      directive: {
        emo_text: '用沐沐温暖的声线',
        ref_audio: 'D:/roles/mumu/ref_neutral.wav',
        ref_text: '早上好呀。',
      },
    })
    await vi.waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith(
        expect.anything(),
        'voice.warm',
        expect.objectContaining({
          profile: 'bundled-cosyvoice2-zh',
          directive: expect.objectContaining({
            ref_audio: 'D:/roles/mumu/ref_neutral.wav',
          }),
        }),
      )
    })
    expect(
      invokeMock.mock.calls.filter(([, method]) => method === 'voice.probe_tts'),
    ).toHaveLength(1)
  })

  it('can retry a background warm after GPU admission defers the first attempt', async () => {
    let warmAttempts = 0
    invokeMock.mockImplementation(async (_id: string, method: string) => {
      if (method === 'voice.probe_tts') {
        return {
          ok: false,
          warmed: false,
          reason: 'gpu_admission_denied',
          retryable: true,
        }
      }
      if (method === 'voice.warm') {
        warmAttempts += 1
        return warmAttempts === 1
          ? { ok: false, reason: 'gpu_admission_denied', retryable: true }
          : { ok: true, warmed: true, sidecar_endpoint: 'http://127.0.0.1:50000' }
      }
      return { ok: false }
    })

    const mod = await import('./useVoiceExpansionWarm')
    mod.resetVoiceExpansionWarmSchedule()
    await mod.scheduleVoiceExpansionWarm(() => false)
    await vi.waitFor(() => expect(warmAttempts).toBe(1))
    await mod.scheduleVoiceExpansionWarm(() => false)
    await vi.waitFor(() => expect(warmAttempts).toBe(2))
  })

  it('keeps sidecar endpoints isolated by TTS profile', async () => {
    invokeMock.mockImplementation(async (_id: string, method: string, params: { profile?: string }) => {
      if (method === 'voice.probe_tts') {
        const port = params.profile === 'profile-a' ? 50101 : 50102
        return { ok: true, warmed: true, sidecar_endpoint: `http://127.0.0.1:${port}` }
      }
      return { ok: false }
    })

    const mod = await import('./useVoiceExpansionWarm')
    mod.resetVoiceExpansionWarmSchedule()
    await expect(
      mod.resolveVoiceSidecarEndpoint('profile-a', '', () => false),
    ).resolves.toBe('http://127.0.0.1:50101')
    await expect(
      mod.resolveVoiceSidecarEndpoint('profile-b', '', () => false),
    ).resolves.toBe('http://127.0.0.1:50102')
    expect(mod.getVoiceSidecarEndpoint('profile-a')).toBe('http://127.0.0.1:50101')
    expect(mod.getVoiceSidecarEndpoint('profile-b')).toBe('http://127.0.0.1:50102')
  })

  it('does not expose an unconfirmed fallback endpoint to direct fetch', async () => {
    invokeMock.mockImplementation(async (_id: string, method: string) => {
      if (method === 'voice.probe_tts') {
        return {
          ok: false,
          reason: 'http_unreachable',
          sidecar_endpoint: 'http://127.0.0.1:50000',
        }
      }
      return { ok: false }
    })

    const mod = await import('./useVoiceExpansionWarm')
    mod.resetVoiceExpansionWarmSchedule()
    await expect(
      mod.resolveVoiceSidecarEndpoint(
        'bundled-cosyvoice2-zh',
        'http://127.0.0.1:50000',
        () => false,
      ),
    ).resolves.toBeNull()
    expect(mod.getVoiceSidecarEndpoint('bundled-cosyvoice2-zh')).toBeNull()
  })
})

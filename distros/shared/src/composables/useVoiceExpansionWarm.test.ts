import { beforeEach, describe, expect, it, vi } from 'vitest'

const mocks = vi.hoisted(() => ({
  invoke: vi.fn(),
  settingsConfig: {
    tts_expansion_enabled: true,
    tts_profile: 'bundled-cosyvoice2-zh',
  } as Record<string, unknown>,
}))

vi.mock('@oclive/shared/api', () => ({
  directoryPluginInvoke: (...args: unknown[]) => mocks.invoke(...args),
  getPluginSettingsUi: vi.fn(async () => ({
    config: mocks.settingsConfig,
  })),
}))

describe('useVoiceExpansionWarm', () => {
  beforeEach(async () => {
    vi.resetModules()
    mocks.invoke.mockReset()
    mocks.settingsConfig = {
      tts_expansion_enabled: true,
      tts_profile: 'bundled-cosyvoice2-zh',
    }
  })

  it('does not expose bundled sidecar until host-admitted warm completes', async () => {
    let warmResolve: ((value: unknown) => void) | null = null
    mocks.invoke.mockImplementation(async (_id: string, method: string) => {
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

    await expect(endpointPromise).resolves.toBeNull()
    expect(warmResolve).not.toBeNull()
    warmResolve?.({ ok: true, sidecar_endpoint: 'http://127.0.0.1:50099' })
    await vi.waitFor(() => {
      expect(mod.getVoiceSidecarEndpoint('bundled-cosyvoice2-zh'))
        .toBe('http://127.0.0.1:50099')
    })
  })

  it('re-admits a warmed bundled sidecar through the host coordinator', async () => {
    mocks.invoke.mockImplementation(async (_id: string, method: string) => {
      if (method === 'voice.probe_tts') {
        return { ok: true, warmed: true, sidecar_endpoint: 'http://127.0.0.1:50000' }
      }
      if (method === 'voice.warm') {
        return {
          ok: true,
          warmed: true,
          already_warmed: true,
          sidecar_endpoint: 'http://127.0.0.1:50000',
        }
      }
      return { ok: false }
    })

    const mod = await import('./useVoiceExpansionWarm')
    mod.resetVoiceExpansionWarmSchedule()
    await mod.scheduleVoiceExpansionWarm(() => false)
    await vi.waitFor(() => {
      expect(mocks.invoke).toHaveBeenCalledWith(
        expect.anything(),
        'voice.warm',
        expect.anything(),
      )
    })
    expect(mod.getVoiceSidecarEndpoint()).toBe('http://127.0.0.1:50000')
  })

  it('prepares a role directive even when the model is already warmed', async () => {
    mocks.invoke.mockImplementation(async (_id: string, method: string) => {
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
      expect(mocks.invoke).toHaveBeenCalledWith(
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
      mocks.invoke.mock.calls.filter(([, method]) => method === 'voice.probe_tts'),
    ).toHaveLength(1)
  })

  it('can retry a background warm after GPU admission defers the first attempt', async () => {
    let warmAttempts = 0
    mocks.invoke.mockImplementation(async (_id: string, method: string) => {
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
    mocks.invoke.mockImplementation(async (_id: string, method: string, params: { profile?: string }) => {
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
    mocks.invoke.mockImplementation(async (_id: string, method: string) => {
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

  it('does not warm when the explicit role policy enables no roles', async () => {
    mocks.settingsConfig = {
      tts_expansion_enabled: true,
      tts_profile: 'bundled-cosyvoice2-zh',
      role_tts_enabled: {},
    }

    const mod = await import('./useVoiceExpansionWarm')
    mod.resetVoiceExpansionWarmSchedule()
    await mod.scheduleVoiceExpansionWarm(() => false)

    expect(mocks.invoke).not.toHaveBeenCalled()
  })
})

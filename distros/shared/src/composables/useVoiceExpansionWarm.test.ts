import { describe, expect, it, vi, beforeEach } from 'vitest'

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
})

import { describe, expect, it } from 'vitest'
import { resolveVoiceTtsRouting } from './voiceTtsRouting'

const globalRouting = {
  tts_profile: 'edge-tts-zh',
  tts_engine: 'edge-tts',
  synth_provider: 'cloud',
  local_synth_endpoint: '',
}

const profiles = new Map([
  ['edge-tts-zh', { engine: 'edge-tts', synth_provider: 'cloud' }],
  [
    'bundled-cosyvoice2-zh',
    {
      engine: 'cosyvoice2',
      synth_provider: 'bundled',
      sidecar_endpoint: 'http://127.0.0.1:50000',
    },
  ],
])

describe('resolveVoiceTtsRouting', () => {
  it('lets roles without an override share the global TTS infrastructure', () => {
    expect(resolveVoiceTtsRouting(globalRouting, undefined, profiles)).toEqual(globalRouting)
  })

  it('applies a role override only to the current speak job', () => {
    expect(
      resolveVoiceTtsRouting(globalRouting, 'bundled-cosyvoice2-zh', profiles),
    ).toEqual({
      tts_profile: 'bundled-cosyvoice2-zh',
      tts_engine: 'cosyvoice2',
      synth_provider: 'bundled',
      local_synth_endpoint: 'http://127.0.0.1:50000',
    })
    expect(globalRouting.tts_profile).toBe('edge-tts-zh')
  })

  it('falls back to the global profile when a role references a missing profile', () => {
    expect(
      resolveVoiceTtsRouting(globalRouting, 'removed-role-profile', profiles),
    ).toEqual(globalRouting)
  })

  it('keeps settings-page provider when the directive only echoes the global profile', () => {
    const globalWithLocalHttp = {
      tts_profile: 'bundled-cosyvoice2-zh',
      tts_engine: 'cosyvoice2',
      synth_provider: 'local_http',
      local_synth_endpoint: 'http://127.0.0.1:9880',
    }
    expect(
      resolveVoiceTtsRouting(globalWithLocalHttp, 'bundled-cosyvoice2-zh', profiles),
    ).toEqual(globalWithLocalHttp)
  })
})

import { describe, expect, it } from 'vitest'
import {
  explicitVoiceRoleTtsDecision,
  hasAnyExplicitVoiceRoleEnabled,
  hasExplicitVoiceRoleTtsPolicy,
  normalizeVoiceRoleTtsEnabled,
} from './voiceRolePolicy'

describe('voice role TTS policy', () => {
  it('treats a missing map as a legacy config', () => {
    expect(hasExplicitVoiceRoleTtsPolicy({ auto_tts: true })).toBe(false)
    expect(explicitVoiceRoleTtsDecision({ auto_tts: true }, 'mumu')).toBeNull()
    expect(hasAnyExplicitVoiceRoleEnabled({ auto_tts: true })).toBeNull()
  })

  it('keeps only explicitly enabled, non-empty role ids', () => {
    const input: Record<string, unknown> = {
      mumu: true,
      disabled: false,
      empty: null,
    }
    input['  moon  '] = true
    expect(normalizeVoiceRoleTtsEnabled(input)).toEqual({
      mumu: true,
      moon: true,
    })
  })

  it('blocks unlisted roles when the explicit policy exists', () => {
    const config = { role_tts_enabled: { mumu: true } }
    expect(explicitVoiceRoleTtsDecision(config, 'mumu')).toBe(true)
    expect(explicitVoiceRoleTtsDecision(config, 'gentle-landlady')).toBe(false)
    expect(hasAnyExplicitVoiceRoleEnabled(config)).toBe(true)
    expect(hasAnyExplicitVoiceRoleEnabled({ role_tts_enabled: {} })).toBe(false)
  })
})

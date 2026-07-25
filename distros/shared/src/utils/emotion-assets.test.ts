import { describe, expect, it } from 'vitest'
import { emotionAssetCandidates } from './emotion-assets'

describe('emotionAssetCandidates', () => {
  it('prefers upgraded mild assets before legacy filenames', () => {
    const candidates = emotionAssetCandidates('happy')
    expect(candidates.indexOf('happy_mild.png')).toBeLessThan(candidates.indexOf('happy.png'))
  })

  it('keeps legacy neutral fallbacks', () => {
    const candidates = emotionAssetCandidates('neutral')
    expect(candidates[0]).toBe('neutral_mild.png')
    expect(candidates).toContain('normal.png')
  })
})

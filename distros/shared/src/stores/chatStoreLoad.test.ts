import { describe, expect, it } from 'vitest'
import { buildSceneLoadCandidates } from './chatStoreLoad'

describe('buildSceneLoadCandidates', () => {
  it('prioritizes backend session scenes, then narrative primary, then pack scenes', () => {
    expect(
      buildSceneLoadCandidates('company', ['home', 'school', 'company'], ['school']),
    ).toEqual(['school', 'company', 'home', 'default'])
  })

  it('deduplicates and always includes home/default fallbacks', () => {
    expect(buildSceneLoadCandidates('home', ['home'], [])).toEqual(['home', 'default'])
  })
})

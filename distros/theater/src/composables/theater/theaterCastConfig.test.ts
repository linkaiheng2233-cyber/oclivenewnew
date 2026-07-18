import type { TheaterSkeleton } from './theaterLogic'
import { describe, expect, it } from 'vitest'
import {
  bindCastToSkeleton,
  DEFAULT_CAST_CONFIG,
  DEFAULT_THEATER_CAST_CONFIG,
  isHybridCast,
  resolveCanonicalReplacementNames,
  resolveCastTier,
  swapCanonicalNamesInBeats,
  swapCanonicalNamesInForks,
} from './theaterCastConfig'

const canonicalSkeleton: TheaterSkeleton = {
  scene: 'breakfast',
  sceneId: 'theater:home',
  cast: {
    a: { roleId: 'mumu', name: '木木', side: 'left' },
    b: { roleId: '枫侵月', name: '枫侵月', side: 'right' },
  },
  beats: [
    {
      id: 'b1',
      cast: 'b',
      name: '枫侵月',
      text: '木木，粥还要不要温一下？',
      stageHint: '把碗推给木木',
    },
    {
      id: 'b2',
      cast: 'a',
      name: '木木',
      text: '……谁要你温了。',
    },
  ],
  forks: {
    tea: [{
      chipId: 'tea',
      insertAfterBeatId: 'b2',
      patchLines: [
        {
          id: 'tea-1',
          cast: 'b',
          name: '枫侵月',
          text: '木木，喝完这杯。',
        },
      ],
    }],
  },
}

describe('theaterCastConfig', () => {
  it('swapCanonicalNamesInBeats replaces names in text and stageHint', () => {
    const next = swapCanonicalNamesInBeats(
      canonicalSkeleton.beats,
      '木木',
      '枫侵月',
      '小木',
      '小枫',
    )
    expect(next[0]?.name).toBe('小枫')
    expect(next[0]?.text).toBe('小木，粥还要不要温一下？')
    expect(next[0]?.stageHint).toBe('把碗推给小木')
    expect(next[1]?.name).toBe('小木')
  })

  it('swapCanonicalNamesInForks updates patchLines', () => {
    const forks = swapCanonicalNamesInForks(
      canonicalSkeleton.forks,
      '木木',
      '枫侵月',
      '小木',
      '小枫',
    )
    const patch = forks.tea?.[0]?.patchLines[0]
    expect(patch?.name).toBe('小枫')
    expect(patch?.text).toBe('小木，喝完这杯。')
  })

  it('bindCastToSkeleton updates cast ids and dialogue without changing beat ids', () => {
    const config = {
      ...DEFAULT_THEATER_CAST_CONFIG,
      castA: { roleId: 'custom-a', displayName: '小木' },
      castB: { roleId: 'custom-b', displayName: '小枫' },
    }
    const runtime = bindCastToSkeleton(canonicalSkeleton, config)
    expect(runtime.cast.a.roleId).toBe('custom-a')
    expect(runtime.cast.b.roleId).toBe('custom-b')
    expect(runtime.beats[0]?.id).toBe('b1')
    expect(runtime.beats[0]?.text).toContain('小木')
    expect(runtime.forks.tea?.[0]?.insertAfterBeatId).toBe('b2')
    expect(runtime.forks.tea?.[0]?.patchLines[0]?.text).toContain('小木')
  })

  it('does not mutate the canonical skeleton input', () => {
    const config = {
      ...DEFAULT_THEATER_CAST_CONFIG,
      castA: { roleId: 'custom-a', displayName: '小木' },
      castB: { roleId: 'custom-b', displayName: '小枫' },
    }
    bindCastToSkeleton(canonicalSkeleton, config)
    expect(canonicalSkeleton.cast.a.roleId).toBe('mumu')
    expect(canonicalSkeleton.beats[0]?.text).toContain('木木')
  })

  it('resolveCastTier treats official pair + family as default', () => {
    expect(resolveCastTier(DEFAULT_THEATER_CAST_CONFIG)).toBe('default')
    expect(resolveCastTier(DEFAULT_CAST_CONFIG)).toBe('default')
    const swappedOfficial = {
      ...DEFAULT_THEATER_CAST_CONFIG,
      castA: { roleId: '枫侵月', displayName: '枫侵月' },
      castB: { roleId: 'mumu', displayName: '沐沐' },
    }
    expect(resolveCastTier(swappedOfficial)).toBe('default')
    const loverOfficial = {
      ...DEFAULT_THEATER_CAST_CONFIG,
      pairRelationId: 'lover' as const,
    }
    expect(resolveCastTier(loverOfficial)).toBe('applied')
    const fullCustom = {
      ...DEFAULT_THEATER_CAST_CONFIG,
      castA: { roleId: 'custom-a', displayName: 'A' },
      castB: { roleId: 'custom-b', displayName: 'B' },
    }
    expect(resolveCastTier(fullCustom)).toBe('applied')
    const hybrid = {
      ...DEFAULT_THEATER_CAST_CONFIG,
      castB: { roleId: 'custom-b', displayName: 'B' },
    }
    expect(resolveCastTier(hybrid)).toBe('applied')
    expect(isHybridCast(hybrid)).toBe(true)
    expect(isHybridCast(fullCustom)).toBe(false)
    expect(isHybridCast(DEFAULT_THEATER_CAST_CONFIG)).toBe(false)
  })

  it('bindCastToSkeleton maps hybrid cast names by slot', () => {
    const hybrid = {
      ...DEFAULT_THEATER_CAST_CONFIG,
      castA: { roleId: 'custom-a', displayName: '小艾' },
      castB: { roleId: 'mumu', displayName: '沐沐' },
    }
    const names = resolveCanonicalReplacementNames(hybrid)
    expect(names.canonicalSideA).toBe('小艾')
    expect(names.canonicalSideB).toBe('沐沐')
    const runtime = bindCastToSkeleton(canonicalSkeleton, hybrid)
    expect(runtime.cast.a.roleId).toBe('custom-a')
    expect(runtime.cast.b.roleId).toBe('mumu')
    expect(runtime.beats[0]?.text).toContain('小艾')
    expect(runtime.beats[0]?.name).toBe('沐沐')
    expect(runtime.beats[1]?.name).toBe('小艾')
  })

  it('bindCastToSkeleton swaps beat sides when official roles are reversed', () => {
    const config = {
      ...DEFAULT_THEATER_CAST_CONFIG,
      castA: { roleId: '枫侵月', displayName: '枫侵月' },
      castB: { roleId: 'mumu', displayName: '沐沐' },
    }
    const runtime = bindCastToSkeleton(canonicalSkeleton, config)
    expect(runtime.cast.a.roleId).toBe('枫侵月')
    expect(runtime.cast.b.roleId).toBe('mumu')
    expect(runtime.beats[0]?.cast).toBe('a')
    expect(runtime.beats[0]?.name).toBe('枫侵月')
    expect(runtime.beats[0]?.text).toContain('沐沐')
    expect(runtime.beats[1]?.cast).toBe('b')
    expect(runtime.beats[1]?.name).toBe('沐沐')
  })
})

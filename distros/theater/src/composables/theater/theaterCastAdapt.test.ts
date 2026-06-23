import { beforeEach, describe, expect, it, vi } from 'vitest'
import {
  applyAdaptedToSkeleton,
  buildRuntimeFromRewrite,
  cacheKeyForCast,
  clearAdaptedCacheForCast,
  clearAllAdaptedCache,
  computeSkeletonHash,
  countAdaptedCacheEntries,
  getAdaptedCache,
  isDefaultCast,
  needsCastAdaptation,
  pruneAdaptedCache,
  resolveCastAdaptStatus,
  setAdaptedCache,
  skeletonToForkTemplates,
} from './theaterCastAdapt'
import { DEFAULT_THEATER_CAST_CONFIG } from './theaterCastConfig'
import type { TheaterSkeleton } from './theaterLogic'

const canonicalSkeleton: TheaterSkeleton = {
  scene: 'breakfast',
  sceneId: 'home',
  cast: {
    a: { roleId: 'mumu', name: '木木', side: 'left' },
    b: { roleId: '枫侵月', name: '枫侵月', side: 'right' },
  },
  beats: [
    { id: 'b1', cast: 'b', name: '枫侵月', text: '开场。' },
    { id: 'b2', cast: 'a', name: '木木', text: '回应。' },
  ],
  forks: {
    tea: [{
      chipId: 'tea',
      insertAfterBeatId: 'b2',
      patchLines: [
        { id: 'tea-1', cast: 'b', name: '枫侵月', text: '罐头。' },
      ],
    }],
  },
}

const customConfig = {
  ...DEFAULT_THEATER_CAST_CONFIG,
  castA: { roleId: 'custom-a', displayName: '小木' },
  castB: { roleId: 'custom-b', displayName: '小枫' },
}

describe('theaterCastAdapt', () => {
  const memoryStore: Record<string, string> = {}

  beforeEach(() => {
    for (const key of Object.keys(memoryStore))
      delete memoryStore[key]
    vi.stubGlobal('localStorage', {
      getItem: (key: string) => memoryStore[key] ?? null,
      setItem: (key: string, value: string) => {
        memoryStore[key] = value
      },
      removeItem: (key: string) => {
        delete memoryStore[key]
      },
    })
  })

  it('isDefaultCast recognizes official mumu × 枫侵月 (any slot order)', () => {
    expect(isDefaultCast(DEFAULT_THEATER_CAST_CONFIG)).toBe(true)
    expect(needsCastAdaptation(DEFAULT_THEATER_CAST_CONFIG)).toBe(false)
    const swappedOfficial = {
      ...DEFAULT_THEATER_CAST_CONFIG,
      castA: { roleId: '枫侵月', displayName: '枫侵月' },
      castB: { roleId: 'mumu', displayName: '沐沐' },
    }
    expect(isDefaultCast(swappedOfficial)).toBe(true)
    expect(needsCastAdaptation(swappedOfficial)).toBe(false)
    expect(isDefaultCast(customConfig)).toBe(false)
    expect(needsCastAdaptation(customConfig)).toBe(true)
  })

  it('default cast with non-family relation needs rewrite', () => {
    const loverDefaultCast = {
      ...DEFAULT_THEATER_CAST_CONFIG,
      pairRelationId: 'lover' as const,
    }
    expect(needsCastAdaptation(loverDefaultCast)).toBe(true)
    expect(resolveCastAdaptStatus(loverDefaultCast, 'breakfast', 'hash')).toBe('renameOnly')
  })

  it('cache key uses theater preset id', () => {
    expect(cacheKeyForCast(DEFAULT_THEATER_CAST_CONFIG, 'breakfast')).toBe('breakfast:mumu:枫侵月:family')
    expect(cacheKeyForCast(
      { ...customConfig, pairRelationId: 'friend' },
      'supermarket',
    )).toBe('supermarket:custom-a:custom-b:friend')
  })

  it('computeSkeletonHash changes when beat ids change', () => {
    const h1 = computeSkeletonHash(canonicalSkeleton)
    const mutated = {
      ...canonicalSkeleton,
      beats: [{ id: 'b9', cast: 'b' as const, name: '枫侵月', text: 'x' }],
    }
    expect(computeSkeletonHash(mutated)).not.toBe(h1)
  })

  it('skeletonToForkTemplates preserves chip and insert anchor', () => {
    const runtime = {
      ...canonicalSkeleton,
      forks: canonicalSkeleton.forks,
    }
    const templates = skeletonToForkTemplates(runtime)
    expect(templates).toHaveLength(1)
    expect(templates[0]?.chip_id).toBe('tea')
    expect(templates[0]?.insert_after_beat_id).toBe('b2')
    expect(templates[0]?.patch_lines[0]?.id).toBe('tea-1')
  })

  it('applyAdaptedToSkeleton merges text but keeps beat/fork ids', () => {
    const sk = { ...canonicalSkeleton }
    const adapted = applyAdaptedToSkeleton(
      sk,
      [
        { id: 'b1', cast: 'b', name: '小枫', text: '适配开场。' },
        { id: 'b2', cast: 'a', name: '小木', text: '适配回应。' },
      ],
      [{
        chip_id: 'tea',
        insert_after_beat_id: 'b2',
        patch_lines: [
          { id: 'tea-1', cast: 'b', name: '小枫', text: '适配罐头。' },
        ],
      }],
    )
    expect(adapted.beats[0]?.text).toBe('适配开场。')
    expect(adapted.beats[0]?.id).toBe('b1')
    expect(adapted.forks.tea?.[0]?.insertAfterBeatId).toBe('b2')
    expect(adapted.forks.tea?.[0]?.patchLines[0]?.text).toBe('适配罐头。')
  })

  it('cache hit/miss and hash invalidation', () => {
    const hash = computeSkeletonHash(canonicalSkeleton)
    expect(getAdaptedCache(customConfig, 'breakfast', hash)).toBeNull()

    const entry = {
      skeletonHash: hash,
      beats: [{ id: 'b1', cast: 'b', name: '小枫', text: 'cached' }],
      forks: [],
      source: 'local',
      ts: Date.now(),
    }
    setAdaptedCache(customConfig, 'breakfast', entry)
    expect(getAdaptedCache(customConfig, 'breakfast', hash)?.beats[0]?.text).toBe('cached')
    expect(getAdaptedCache(customConfig, 'breakfast', 'other-hash')).toBeNull()
  })

  it('pruneAdaptedCache drops oldest entries by ts', () => {
    const hash = computeSkeletonHash(canonicalSkeleton)
    const store: Record<string, { skeletonHash: string, beats: [], forks: [], source: string, ts: number }> = {}
    for (let i = 0; i < 10; i++) {
      store[`supermarket:custom-a:custom-b-${i}`] = {
        skeletonHash: hash,
        beats: [],
        forks: [],
        source: 'local',
        ts: i + 1,
      }
    }
    memoryStore['oclive.theater.adapted.v2'] = JSON.stringify(store)
    pruneAdaptedCache(8)
    const parsed = JSON.parse(memoryStore['oclive.theater.adapted.v2']!) as Record<string, { ts: number }>
    expect(Object.keys(parsed)).toHaveLength(8)
    expect(parsed['supermarket:custom-a:custom-b-0']).toBeUndefined()
    expect(parsed['supermarket:custom-a:custom-b-1']).toBeUndefined()
    expect(parsed['supermarket:custom-a:custom-b-9']).toBeDefined()
  })

  it('clearAdaptedCacheForCast removes one cast combination', () => {
    const hash = computeSkeletonHash(canonicalSkeleton)
    setAdaptedCache(customConfig, 'breakfast', {
      skeletonHash: hash,
      beats: [],
      forks: [],
      source: 'local',
      ts: 1,
    })
    clearAdaptedCacheForCast(customConfig, 'breakfast')
    expect(getAdaptedCache(customConfig, 'breakfast', hash)).toBeNull()
  })

  it('clearAllAdaptedCache removes every entry', () => {
    const hash = computeSkeletonHash(canonicalSkeleton)
    setAdaptedCache(customConfig, 'breakfast', {
      skeletonHash: hash,
      beats: [],
      forks: [],
      source: 'local',
      ts: 1,
    })
    setAdaptedCache(
      { ...customConfig, castB: { roleId: 'other-b', displayName: 'B2' } },
      'breakfast',
      { skeletonHash: hash, beats: [], forks: [], source: 'local', ts: 2 },
    )
    expect(countAdaptedCacheEntries()).toBe(2)
    expect(clearAllAdaptedCache()).toBe(2)
    expect(countAdaptedCacheEntries()).toBe(0)
    expect(clearAllAdaptedCache()).toBe(0)
  })

  it('buildRuntimeFromRewrite replaces beats and forks', () => {
    const baseline = { ...canonicalSkeleton }
    const rewritten = buildRuntimeFromRewrite(
      baseline,
      [
        { id: 'b1', cast: 'a', name: '小木', text: '全新开场。' },
        { id: 'b2', cast: 'b', name: '小枫', text: '全新回应。' },
      ],
      [{
        chip_id: 'tea',
        insert_after_beat_id: 'b1',
        patch_lines: [
          { id: 'tea-1', cast: 'b', name: '小枫', text: '新 fork。' },
        ],
      }],
    )
    expect(rewritten.beats).toHaveLength(2)
    expect(rewritten.beats[0]?.text).toBe('全新开场。')
    expect(rewritten.forks.tea?.[0]?.insertAfterBeatId).toBe('b1')
    expect(rewritten.forks.tea?.[0]?.patchLines[0]?.text).toBe('新 fork。')
  })

  it('default cast does not require adaptation cache', () => {
    expect(isDefaultCast(DEFAULT_THEATER_CAST_CONFIG)).toBe(true)
    expect(getAdaptedCache(DEFAULT_THEATER_CAST_CONFIG, 'breakfast', 'any')).toBeNull()
  })

  it('resolveCastAdaptStatus returns default, cached, or renameOnly', () => {
    const hash = computeSkeletonHash(canonicalSkeleton)
    expect(resolveCastAdaptStatus(DEFAULT_THEATER_CAST_CONFIG, 'breakfast', hash)).toBe('default')
    expect(resolveCastAdaptStatus(customConfig, 'breakfast', hash)).toBe('renameOnly')

    setAdaptedCache(customConfig, 'breakfast', {
      skeletonHash: hash,
      beats: [{ id: 'b1', cast: 'b', name: '小枫', text: 'cached' }],
      forks: [],
      source: 'local',
      ts: Date.now(),
    })
    expect(resolveCastAdaptStatus(customConfig, 'breakfast', hash)).toBe('cached')
    expect(resolveCastAdaptStatus(customConfig, 'breakfast', 'stale-hash')).toBe('renameOnly')
  })
})
